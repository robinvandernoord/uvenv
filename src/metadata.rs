use crate::helpers::ResultToString;
use crate::symlinks::check_symlink;
use crate::uv::{uv_get_installed_version, uv_venv, Helpers};
use itertools::Itertools;
use owo_colors::OwoColorize;
use pep508_rs::Requirement;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use tokio::fs::File;
use tokio::io::AsyncReadExt;
use tokio::io::AsyncWriteExt;
use uv_interpreter::PythonEnvironment;

const BIN_DIR: &str = ".local/bin";
const WORK_DIR: &str = ".local/uvx";
const INDENT: &str = "    ";

// tells 'file' that a .metadata file is 'data' (instead of making it guess)
//                           U     V     X    SOH  version(2)  STX (padding):
const MAGIC_HEADER: &[u8] = &[0x55, 0x56, 0x58, 0x01, 0x32, 0x04, 0x00]; // hex, 7 bytes

pub fn get_home_dir() -> PathBuf {
    home::home_dir().expect("Failed to get home directory")
}

pub fn get_bin_dir() -> PathBuf {
    let home_dir = get_home_dir();
    home_dir.join(BIN_DIR)
}

pub fn get_work_dir() -> PathBuf {
    let home_dir = get_home_dir();
    home_dir.join(WORK_DIR)
}

pub fn get_venv_dir() -> PathBuf {
    let work_dir = get_work_dir();
    work_dir.join("venvs")
}

pub fn venv_path(venv_name: &str) -> PathBuf {
    get_venv_dir().join(venv_name)
}

#[derive(Debug, PartialEq, Deserialize, Serialize)] // dbg_pls::DebugPls
pub struct Metadata {
    // order is important!!
    pub name: String,
    #[serde(default)]
    pub scripts: HashMap<String, bool>,
    pub install_spec: String,
    #[serde(default)]
    pub extras: HashSet<String>,
    #[serde(default)]
    pub requested_version: String,
    pub installed_version: String,
    pub python: String,
    pub python_raw: String,
    #[serde(default)]
    pub injected: HashSet<String>,
    #[serde(default)]
    pub editable: bool,
}

impl Metadata {
    pub fn new(name: &str) -> Metadata {
        Metadata {
            name: name.to_string(),
            scripts: HashMap::new(),
            install_spec: name.to_string(),
            extras: HashSet::new(),
            requested_version: String::new(),
            installed_version: String::new(),
            python: String::new(),
            python_raw: String::new(),
            injected: HashSet::new(),
            editable: false,
        }
    }

    pub fn find(req: &Requirement) -> Metadata {
        let mut empty = Metadata::new(req.name.as_ref());

        empty.installed_version = uv_get_installed_version(&req.name, None).unwrap_or_default();

        empty.fill(None);
        empty
    }

    /// try to guess/deduce some values
    pub fn fill(
        &mut self,
        maybe_venv: Option<&PythonEnvironment>,
    ) -> Option<()> {
        let _v: PythonEnvironment;

        let venv = match maybe_venv {
            Some(v) => v,
            None => match uv_venv(None) {
                Some(v) => {
                    _v = v;
                    &_v
                },
                None => return None,
            },
        };

        let python_info = venv.interpreter().markers();

        if self.install_spec.is_empty() {
            self.install_spec = String::from(&self.name);
        }

        if self.python.is_empty() {
            self.python = format!(
                "{} {}",
                python_info.platform_python_implementation, python_info.python_full_version
            )
        }

        if self.python_raw.is_empty() {
            self.python_raw = venv.stdlib_as_string();
        }

        Some(())
    }

    pub async fn for_dir(
        dirname: &Path,
        recheck_scripts: bool,
    ) -> Option<Metadata> {
        let meta_path = dirname.join(".metadata");

        Metadata::for_file(&meta_path, recheck_scripts).await
    }

    pub async fn for_requirement(
        requirement: &Requirement,
        recheck_scripts: bool,
    ) -> Metadata {
        let requirement_name = requirement.name.to_string();
        let venv_dir = venv_path(&requirement_name);

        match Metadata::for_dir(&venv_dir, recheck_scripts).await {
            Some(m) => m,
            None => Metadata::find(requirement),
        }
    }

    pub async fn for_file(
        filename: &Path,
        recheck_scripts: bool,
    ) -> Option<Metadata> {
        let result = load_metadata(filename, recheck_scripts).await;
        result.ok()
    }

    pub async fn save(
        &self,
        dirname: &Path,
    ) -> Result<(), String> {
        let meta_path = dirname.join(".metadata");
        store_metadata(&meta_path, self).await
    }

    pub async fn check_scripts(
        &mut self,
        venv_path: &Path,
    ) {
        for (key, value) in self.scripts.iter_mut() {
            *value = check_symlink(key, venv_path).await;
        }
    }

    pub fn format_short(&self) -> String {
        format!("- {} {}", self.name, self.installed_version.cyan())
    }

    #[allow(dead_code)]
    pub fn vec_extras(&self) -> Vec<&str> {
        self.extras.iter().map(|k| k.as_ref()).collect()
    }

    pub fn vec_injected(&self) -> Vec<&str> {
        self.injected.iter().map(|k| k.as_ref()).collect()
    }

    pub fn format_extras(&self) -> String {
        self.extras
            .iter()
            .map(|k| format!("'{}'", k.green()))
            .join(",")
    }

    pub fn format_injected(&self) -> String {
        self.injected
            .iter()
            .map(|k| format!("'{}'", k.green()))
            .join(", ")
    }

    pub fn format_human(&self) -> String {
        let mut result = format!("- {}", self.name);

        if !self.extras.is_empty() {
            result.push_str(&format!("[{}]", self.format_extras()));
        }

        if !self.requested_version.is_empty() {
            result.push_str(&format!(" {}", self.requested_version.cyan()));
        }

        if self.editable {
            result.push_str(&format!(" {}", "--editable".yellow()))
        }

        result.push('\n');

        result.push_str(&format!(
            "{}Installed Version: {} on {}.\n",
            INDENT,
            self.installed_version.cyan(),
            self.python.bright_blue()
        ));

        let formatted_injects = self.format_injected();
        result.push_str(&format!(
            "{}Injected Packages: {}\n",
            INDENT, formatted_injects
        ));

        let formatted_scripts = self
            .scripts
            .iter()
            .map(|(key, value)| {
                if *value {
                    key.green().to_string()
                } else {
                    key.red().to_string()
                }
            })
            .join(" | ");
        result.push_str(&format!("{}Scripts: {}", INDENT, formatted_scripts));

        result
    }
}

/// Drop the MAGIC_HEADER from a buffer (if present)
pub fn strip_header(buf: &mut Vec<u8>) {
    // postponed: the current header is 7 chars long.
    //            the version should be ignored for starts_with,
    //            and if the length should change, this must be 'match'ed based on the version
    if buf.starts_with(MAGIC_HEADER) {
        let _ = buf.drain(0..MAGIC_HEADER.len());
    }
}

/// Prepend the MAGIC_HEADER to a buffer
fn add_header(buf: &mut Vec<u8>) {
    let mut new_buf = Vec::with_capacity(MAGIC_HEADER.len() + buf.len());
    new_buf.extend_from_slice(MAGIC_HEADER);
    new_buf.append(buf);
    *buf = new_buf;
}

pub async fn load_metadata(
    filename: &Path,
    recheck_scripts: bool,
) -> Result<Metadata, String> {
    // Open the msgpack file
    let mut file = File::open(filename).await.map_err_to_string()?;

    let mut buf = Vec::new();
    file.read_to_end(&mut buf).await.map_err_to_string()?;

    strip_header(&mut buf);

    // Read the contents of the file into a Metadata struct
    let mut metadata: Metadata = rmp_serde::decode::from_slice(&buf[..]).map_err_to_string()?;

    if recheck_scripts {
        if let Some(folder) = filename.parent() {
            metadata.check_scripts(folder).await
        }
    }

    Ok(metadata)
}

pub async fn store_metadata(
    filename: &Path,
    metadata: &Metadata,
) -> Result<(), String> {
    // Open the msgpack file
    let mut file = File::create(filename).await.map_err_to_string()?;

    // Read the contents of the file into a Metadata struct
    let mut bytes = rmp_serde::encode::to_vec(metadata).map_err_to_string()?;

    add_header(&mut bytes);

    file.write_all(&bytes).await.map_err_to_string()?;

    Ok(())
}
