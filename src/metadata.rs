use crate::helpers::ResultToString;
use crate::symlinks::check_symlink;
use itertools::Itertools;
use owo_colors::OwoColorize;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::fs::File;
use std::io::Write;
use std::path::{Path, PathBuf};

const BIN_DIR: &'static str = ".local/bin";
const WORK_DIR: &'static str = ".local/uvx";
const INDENT: &'static str = "    ";

pub fn get_home_dir() -> PathBuf {
    return home::home_dir().expect("Failed to get home directory");
}

pub fn get_bin_dir() -> PathBuf {
    let home_dir = get_home_dir();
    return home_dir.join(BIN_DIR);
}

pub fn get_work_dir() -> PathBuf {
    let home_dir = get_home_dir();
    return home_dir.join(WORK_DIR);
}

pub fn get_venv_dir() -> PathBuf {
    let work_dir = get_work_dir();
    return work_dir.join("venvs");
}

#[derive(Debug, PartialEq, Deserialize, Serialize, dbg_pls::DebugPls)]
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
}

impl Metadata {
    pub fn new(name: &str) -> Metadata {
        return Metadata {
            name: name.to_string(),
            scripts: HashMap::new(),
            install_spec: name.to_string(),
            extras: HashSet::new(),
            requested_version: String::new(),
            installed_version: String::new(),
            python: String::new(),
            python_raw: String::new(),
            injected: HashSet::new(),
        };
    }

    pub fn for_dir(dirname: &PathBuf) -> Option<Metadata> {
        let meta_path = dirname.join(".metadata");

        return Metadata::for_file(&meta_path);
    }

    pub fn for_file(filename: &PathBuf) -> Option<Metadata> {
        return load_metadata(filename).ok();
    }

    pub fn save(&self, dirname: &PathBuf) -> Result<(), String> {
        let meta_path = dirname.join(".metadata");
        return store_metadata(&meta_path, &self);
    }

    pub fn check_scripts(&mut self, venv_path: &Path) {
        for (key, value) in self.scripts.iter_mut() {
            *value = check_symlink(key, venv_path);
        }
    }

    pub fn format_short(&self) -> String {
        return format!("- {} {}", self.name, self.installed_version.cyan());
    }

    pub fn format_human(&self) -> String {
        let mut result = format!("- {}", self.name); // todo: colorized extra's (+ install spec?)

        if self.extras.len() > 0 {
            result.push_str(&format!(
                "[{}]",
                self.extras
                    .iter()
                    .map(|k| format!("'{}'", k.green()))
                    .join(",")
            ));
        }

        if self.requested_version.len() > 0 {
            result.push_str(&format!(" {}", self.requested_version.cyan()));
        }

        result.push_str("\n");

        result.push_str(&format!(
            "{}Installed Version: {} on {}.\n",
            INDENT,
            self.installed_version.cyan(),
            self.python.bright_blue()
        ));

        let formatted_injects = self
            .injected
            .iter()
            .map(|k| format!("'{}'", k.green()))
            .join(", ");
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

        return result;
    }
}

pub fn load_metadata(filename: &Path) -> Result<Metadata, String> {
    // Open the msgpack file
    let file = File::open(filename).map_err_to_string()?;

    // Read the contents of the file into a Metadata struct
    let mut metadata: Metadata = rmp_serde::decode::from_read(file).map_err_to_string()?;

    // fixme: make this optional or move it:
    let _ = &metadata.check_scripts(&filename.parent().unwrap());

    Ok(metadata)
}

pub fn store_metadata(filename: &Path, metadata: &Metadata) -> Result<(), String> {
    // Open the msgpack file
    let mut file = File::create(filename).map_err_to_string()?;

    // Read the contents of the file into a Metadata struct
    let bytes = rmp_serde::encode::to_vec(metadata).map_err_to_string()?;

    file.write_all(&bytes).map_err_to_string()?;

    Ok(())
}
