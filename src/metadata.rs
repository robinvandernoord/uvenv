use crate::pypi::get_latest_version;
use crate::symlinks::check_symlink;
use crate::uv::{uv_get_installed_version, uv_venv, Helpers};
use anyhow::anyhow;
use itertools::Itertools;
use owo_colors::OwoColorize;
use pep440_rs::{Version, VersionSpecifier};
use pep508_rs::Requirement;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::fs::remove_dir_all;
use std::path::{Path, PathBuf};
use std::str::FromStr;
use tokio::fs::{create_dir_all, File};
use tokio::io::AsyncReadExt;
use tokio::io::AsyncWriteExt;
use uv_toolchain::PythonEnvironment;

const BIN_DIR: &str = ".local/bin";
const WORK_DIR: &str = ".local/uvx";
const INDENT: &str = "    ";

// tells 'file' that a .metadata file is 'data' (instead of making it guess)
//                           U     V     X    SOH  version(2)  STX (padding):
const MAGIC_HEADER: &[u8] = &[0x55, 0x56, 0x58, 0x01, 0x32, 0x04, 0x00]; // hex, 7 bytes

pub fn get_home_dir() -> PathBuf {
    if cfg!(test) {
        let test_dir = std::env::temp_dir().join("uvx-test");
        // fixme:
        let _ = dbg!(remove_dir_all(&test_dir));
        let _ = dbg!(std::fs::create_dir_all(&test_dir));
        test_dir
    } else {
        home::home_dir().expect("Failed to get home directory")
    }
}

pub fn get_bin_dir() -> PathBuf {
    let home_dir = get_home_dir();
    home_dir.join(BIN_DIR)
}

pub async fn ensure_bin_dir() -> PathBuf {
    let bin_dir = get_bin_dir();

    if !bin_dir.exists() {
        let _ = create_dir_all(&bin_dir).await;
    }

    bin_dir
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

pub fn version_0() -> Version {
    Version::from_str("0.0.0").expect("Version 0.0.0 should be parseable.")
}

#[derive(Debug, PartialEq, Eq, Deserialize, Serialize)] // dbg_pls::DebugPls
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
    #[serde(default)]
    pub available_version: String,
    #[serde(default)]
    pub outdated: bool,
}

impl Metadata {
    pub fn new(name: &str) -> Self {
        Self {
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
            available_version: String::new(),
            outdated: false,
        }
    }

    #[allow(dead_code)]
    pub fn requested_version_parsed(&self) -> Version {
        Version::from_str(&self.requested_version).unwrap_or_else(|_| version_0())
    }
    pub fn installed_version_parsed(&self) -> Version {
        Version::from_str(&self.installed_version).unwrap_or_else(|_| version_0())
    }

    pub fn find(req: &Requirement) -> Self {
        let mut empty = Self::new(req.name.as_ref());

        empty.installed_version = uv_get_installed_version(&req.name, None).unwrap_or_default();

        empty.fill(None);
        empty
    }

    pub fn fill_python(
        &mut self,
        venv: &PythonEnvironment,
    ) {
        let python_info = venv.interpreter().markers();

        if self.python.is_empty() {
            self.python = format!(
                "{} {}",
                python_info.platform_python_implementation(),
                python_info.python_full_version()
            );
        }

        if self.python_raw.is_empty() {
            self.python_raw = venv.stdlib_as_string();
        }
    }

    /// try to guess/deduce some values
    pub fn fill(
        &mut self,
        maybe_venv: Option<&PythonEnvironment>,
    ) -> Option<()> {
        let environment: PythonEnvironment;

        if self.install_spec.is_empty() {
            self.install_spec = String::from(&self.name);
        }

        let venv = match maybe_venv {
            Some(v) => v,
            None => match uv_venv(None) {
                Ok(venv) => {
                    // black magic fuckery to
                    // get a reference to the currently active venv
                    environment = venv;
                    &environment
                },
                Err(_) => return None,
            },
        };

        self.fill_python(venv);

        Some(())
    }

    /// like `for_dir` but with an owned dirname Pathbuf instead of &Path
    /// (required to work with Futures) -> also returns a Result which is more useful with a future
    pub async fn for_owned_dir(
        dirname: PathBuf,
        config: &LoadMetadataConfig,
    ) -> anyhow::Result<Self> {
        let meta_path = dirname.join(".metadata");

        Self::for_file(&meta_path, config).await.map_or_else(
            || {
                let venv_name = dirname
                    .file_name()
                    .and_then(|fname| fname.to_str())
                    .unwrap_or_default();

                Err(anyhow!(
                    "Metadata for '{}' could not be loaded.",
                    venv_name.red()
                ))
            },
            Ok,
        )
    }

    pub async fn for_dir(
        dirname: &Path,
        config: &LoadMetadataConfig,
    ) -> Option<Self> {
        let meta_path = dirname.join(".metadata");

        Self::for_file(&meta_path, config).await
    }

    pub async fn for_requirement(
        requirement: &Requirement,
        config: &LoadMetadataConfig,
    ) -> Self {
        let requirement_name = requirement.name.to_string();
        let venv_dir = venv_path(&requirement_name);

        Self::for_dir(&venv_dir, config)
            .await
            .map_or_else(|| Self::find(requirement), |meta| meta)
    }

    pub async fn for_file(
        filename: &Path,
        config: &LoadMetadataConfig,
    ) -> Option<Self> {
        let result = load_metadata(filename, config).await;
        result.ok()
    }

    pub async fn save(
        &self,
        dirname: &Path,
    ) -> anyhow::Result<()> {
        let meta_path = dirname.join(".metadata");
        store_metadata(&meta_path, self).await
    }

    pub async fn check_for_update(
        &mut self,
        prereleases: bool,
        ignore_constraints: bool,
    ) {
        let constraint = if ignore_constraints || self.requested_version.is_empty() {
            None
        } else {
            VersionSpecifier::from_str(&self.requested_version).ok()
        };

        if let Some(latest_version) = get_latest_version(&self.name, !prereleases, constraint).await
        {
            let installed_version = self.installed_version_parsed();
            self.available_version = latest_version.to_string();
            self.outdated = latest_version > installed_version;
        }
    }

    pub async fn check_scripts(
        &mut self,
        venv_path: &Path,
    ) {
        for (key, value) in &mut self.scripts {
            *value = check_symlink(key, venv_path).await;
        }
    }

    pub fn invalid_scripts(&self) -> Vec<String> {
        let list: Vec<String> = self
            .scripts
            .iter()
            //                                if True, the script is valid -> skip from filter_map
            .filter_map(|(k, v)| if *v { None } else { Some(k.to_owned()) })
            .collect();

        list
    }

    pub fn format_installed_version(&self) -> String {
        if self.outdated {
            self.installed_version.red().to_string()
        } else {
            self.installed_version.cyan().to_string()
        }
    }

    pub fn format_short(&self) -> String {
        format!("- {} {}", self.name, self.format_installed_version())
    }

    #[allow(dead_code)]
    pub fn vec_extras(&self) -> Vec<&str> {
        self.extras.iter().map(AsRef::as_ref).collect()
    }

    pub fn vec_injected(&self) -> Vec<&str> {
        self.injected.iter().map(AsRef::as_ref).collect()
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
            result.push_str(&format!(" {}", "--editable".yellow()));
        }

        result.push('\n');

        result.push_str(&format!(
            "{}Installed Version: {} on {}.\n",
            INDENT,
            self.format_installed_version(),
            self.python.bright_blue()
        ));

        if self.outdated && !self.available_version.is_empty() {
            result.push_str(&format!(
                "{}Available Version: {}.\n",
                INDENT,
                self.available_version.green(),
            ));
        }

        if !self.injected.is_empty() {
            let formatted_injects = self.format_injected();
            result.push_str(&format!("{INDENT}Injected Packages: {formatted_injects}\n"));
        }

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
        result.push_str(&format!("{INDENT}Scripts: {formatted_scripts}"));

        result
    }
}

/// Drop the `MAGIC_HEADER` from a buffer (if present)
pub fn strip_header(buf: &mut Vec<u8>) {
    // postponed: the current header is 7 chars long.
    //            the version should be ignored for starts_with,
    //            and if the length should change, this must be 'match'ed based on the version
    if buf.starts_with(MAGIC_HEADER) {
        let _ = buf.drain(0..MAGIC_HEADER.len());
    }
}

/// Prepend the `MAGIC_HEADER` to a buffer
fn add_header(buf: &mut Vec<u8>) {
    let mut new_buf = Vec::with_capacity(MAGIC_HEADER.len() + buf.len());
    new_buf.extend_from_slice(MAGIC_HEADER);
    new_buf.append(buf);
    *buf = new_buf;
}

/// 'buf' is required to hold the data internally, with the same lifetime as the unserialized object
/// Mimimal example:
///
///
///     pub async fn load_setup_metadata(filename: &Path) -> anyhow::Result<SetupMetadata> {
///       let mut buf = Vec::new(); // allocate memory for the object
///
///       // Open the msgpack file
///       let metadata: SetupMetadata = load_generic_msgpack(filename, &mut buf).await?;
///
///       Ok(metadata)
///     }
pub async fn load_generic_msgpack<'a, T: serde::Deserialize<'a>>(
    filename: &Path,
    buf: &'a mut Vec<u8>,
) -> anyhow::Result<T> {
    // Open the msgpack file
    let mut file = File::open(filename).await?;

    // for some reason (I guess the lifetime), buf can just be passed around now?
    file.read_to_end(buf).await?;
    strip_header(buf);

    // Read the contents of the file into a Metadata struct
    let metadata: T = rmp_serde::decode::from_slice(&buf[..])?;

    Ok(metadata)
}

#[allow(clippy::struct_excessive_bools)]
pub struct LoadMetadataConfig {
    pub recheck_scripts: bool,
    pub updates_check: bool,
    pub updates_prereleases: bool,
    pub updates_ignore_constraints: bool,
}

impl LoadMetadataConfig {
    #[allow(dead_code)]
    pub const fn all() -> Self {
        Self {
            recheck_scripts: true,
            updates_check: true,
            updates_prereleases: true,
            updates_ignore_constraints: true,
        }
    }

    pub const fn none() -> Self {
        Self {
            recheck_scripts: false,
            updates_check: false,
            updates_prereleases: false,
            updates_ignore_constraints: false,
        }
    }

    pub const fn default() -> Self {
        Self {
            recheck_scripts: true,
            updates_check: true,
            updates_prereleases: false,
            updates_ignore_constraints: false,
        }
    }
}

pub async fn load_metadata(
    filename: &Path,
    config: &LoadMetadataConfig,
) -> anyhow::Result<Metadata> {
    let mut buf = Vec::new();

    let mut metadata: Metadata = load_generic_msgpack(filename, &mut buf).await?;

    if let Some(folder) = filename.parent() {
        // filename.parent should always be Some

        if config.recheck_scripts {
            metadata.check_scripts(folder).await;
        }

        if config.updates_check {
            metadata
                .check_for_update(
                    config.updates_prereleases,
                    config.updates_ignore_constraints,
                )
                .await;
        }
    }

    Ok(metadata)
}

pub async fn store_generic_msgpack<T: serde::Serialize>(
    filename: &Path,
    metadata: &T,
) -> anyhow::Result<()> {
    // Open the msgpack file
    let mut file = File::create(filename).await?;

    // Read the contents of the file into a Metadata struct
    let mut bytes = rmp_serde::encode::to_vec(metadata)?;

    add_header(&mut bytes);

    file.write_all(&bytes).await?;

    Ok(())
}

pub async fn store_metadata(
    filename: &Path,
    metadata: &Metadata,
) -> anyhow::Result<()> {
    store_generic_msgpack(filename, metadata).await
}
