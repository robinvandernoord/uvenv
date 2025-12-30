use crate::commands::self_update::extract_version;
use crate::pypi::get_latest_version;
use crate::symlinks::check_symlink;
use crate::uv::{Helpers, PythonSpecifier, uv_freeze, uv_get_installed_version, uv_venv};
use core::cmp::Ordering;
use core::fmt::Write;
use core::str::FromStr;
use itertools::Itertools;
use owo_colors::OwoColorize;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashSet};
use std::path::{Path, PathBuf};
use tokio::fs::{File, create_dir_all};
use tokio::io::AsyncReadExt;
use tokio::io::AsyncWriteExt;
use uv_pep440::{Version, VersionSpecifier};
use uv_pep508::Requirement;
use uv_python::PythonEnvironment;

const INDENT: &str = "    ";

// tells 'file' that a .metadata file is 'data' (instead of making it guess)
//                           U     V     X    SOH  version(2)  STX (padding):
const MAGIC_HEADER_V2: &[u8] = &[0x55, 0x56, 0x58, 0x01, 0x32, 0x04, 0x00]; // hex, 7 bytes

pub fn get_home_dir() -> PathBuf {
    // if snap: $HOME is now ~/snap/uvenv/<revision>/
    home::home_dir().expect("Failed to get home directory")
}

pub fn get_bin_dir() -> PathBuf {
    let home_dir = get_home_dir();

    if cfg!(feature = "snap") {
        // $SNAP/bin, which exists in snap's $PATH
        home_dir.join("bin")
    } else {
        home_dir.join(".local/bin")
    }
}

pub async fn ensure_bin_dir() -> PathBuf {
    let bin_dir = get_bin_dir();

    if !bin_dir.exists() {
        // may fail I guess:
        let _ = create_dir_all(&bin_dir).await;
    }

    bin_dir
}

pub fn get_work_dir() -> PathBuf {
    let home_dir = get_home_dir();
    if cfg!(feature = "snap") {
        // on snap, store everything in the 'common' ($SNAP_USER_COMMON)
        // dir because that's scoped to the snap and not changed across revisions
        std::env::var("SNAP_USER_COMMON")
            .map(PathBuf::from)
            .unwrap_or(home_dir)
    } else {
        home_dir.join(".local/uvenv")
    }
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

#[derive(Debug)]
pub enum FileType {
    Shebang { interpreter: String },
    Binary,
    Unusable(String),
}

async fn identify_file(path: &Path) -> anyhow::Result<FileType> {
    if !path.exists() {
        return Ok(FileType::Unusable("No Such File".to_owned()));
    }

    if path.is_dir() {
        return Ok(FileType::Unusable("Directory".to_owned()));
    }

    let mut file = File::open(path).await?;
    let mut buffer = [0_u8; 1024];
    let n = file.read(&mut buffer).await?;

    if n == 0 {
        return Ok(FileType::Unusable("Empty".to_owned()));
    }

    let bytes = &buffer.get(..n).unwrap_or_default();

    // Check for shebang (#!)
    if bytes.starts_with(b"#!") {
        let interpreter = String::from_utf8_lossy(bytes)
            .lines()
            .next()
            .unwrap_or("")
            .trim()
            .to_owned();

        return Ok(FileType::Shebang { interpreter });
    }

    // Check if file is binary
    let is_binary = bytes.iter().any(|&byte| byte == 0 || byte > 127);
    if is_binary {
        return Ok(FileType::Binary);
    }

    Ok(FileType::Unusable("No Shebang".to_owned()))
}

pub fn check_shebang(shebang: &str) -> bool {
    let Some(stripped_path) = shebang.strip_prefix("#!") else {
        return false;
    };
    let interpreter_path = Path::new(stripped_path);

    // should chmod also be checked?
    interpreter_path.exists()
}

pub async fn check_executable(script_name: &str) -> Option<String> {
    // todo
    let script_path = ensure_bin_dir().await.join(script_name);
    let Ok(result) = identify_file(&script_path).await else {
        return Some("Could not identify script type".to_owned());
    };
    match result {
        FileType::Shebang { interpreter } => {
            if check_shebang(&interpreter) {
                None
            } else {
                Some("Interpreter does not exist".to_owned())
            }
        },
        FileType::Binary => {
            // should chmod also be checked?
            None
        },
        FileType::Unusable(reason) => Some(reason),
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct Metadata {
    // order is important!!
    pub name: String,
    #[serde(default)]
    pub scripts: BTreeMap<String, bool>,
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
    #[serde(default)]
    pub python_spec: Option<String>,
}

impl PartialOrd for Metadata {
    fn partial_cmp(
        &self,
        other: &Self,
    ) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Metadata {
    fn cmp(
        &self,
        other: &Self,
    ) -> Ordering {
        String::cmp(&self.name, &other.name)
    }
}

impl Metadata {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_owned(),
            scripts: BTreeMap::new(),
            install_spec: name.to_owned(),
            python_spec: None,
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

    // /// When using uvenv's install.sh, the installation is done via bash
    // /// so the metadata msgpack is not stored. This function works as a fallback.
    // fn for_self_install() -> Self {
    //     Self {
    //         name: "uvenv".to_owned(),
    //         ..Default::default()
    //     }
    // }

    /// return `PythonEnvironment` (if available) for this metadata
    pub fn venv(&self) -> Option<PythonEnvironment> {
        let venv_dir = venv_path(&self.name);

        PythonSpecifier::PathBuf(&venv_dir).into_environment().ok()
    }

    #[expect(dead_code, reason = "Useful variant of `installed_version_parsed`")]
    pub fn requested_version_parsed(&self) -> Version {
        Version::from_str(&self.requested_version).unwrap_or_else(|_| version_0())
    }

    pub fn installed_version_parsed(&self) -> Version {
        Version::from_str(&self.installed_version).unwrap_or_else(|_| version_0())
    }

    // check actual currently installed version
    pub fn find_currently_installed_version_raw(&self) -> Option<String> {
        let venv = self.venv()?;
        let frozen = uv_freeze(&venv).ok()?;

        extract_version(&frozen, &self.name)
    }
    pub fn find_currently_installed_version(&self) -> Option<Version> {
        let version_raw = self.find_currently_installed_version_raw()?;
        Version::from_str(&version_raw).ok()
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
            Some(venv) => venv,
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
    ) -> Option<Self> {
        let meta_path = dirname.join(".metadata");

        Self::for_file(&meta_path, config).await.map_or_else(
            || {
                let venv_name = dirname
                    .file_name()
                    .and_then(|fname| fname.to_str())
                    .unwrap_or_default();

                if venv_name != "uvenv" {
                    // exception case for uvenv itself
                    eprintln!("Metadata for '{}' could not be loaded.", venv_name.red());
                }

                None
            },
            Some,
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
            .unwrap_or_else(|| Self::find(requirement))
    }

    pub async fn for_file(
        filename: &Path,
        config: &LoadMetadataConfig,
    ) -> Option<Self> {
        load_metadata(filename, config).await.ok()
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
            let installed_version = self
                // first try getting latest from venv:
                .find_currently_installed_version()
                // fallback to cached version:
                .unwrap_or_else(|| self.installed_version_parsed());

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
        self
            .scripts
            .iter()
            //                                if True, the script is valid -> skip from filter_map
            .filter_map(|(key, val)| if *val { None } else { Some(key.to_owned()) })
            .collect()
    }

    pub async fn broken_scripts(&self) -> Vec<String> {
        let mut result = vec![];

        for script in self.scripts.keys() {
            if let Some(reason) = check_executable(script).await {
                result.push(reason);
            }
        }

        result
    }

    pub fn format_installed_version(&self) -> String {
        if self.outdated {
            self.installed_version.red().to_string()
        } else {
            self.installed_version.cyan().to_string()
        }
    }

    // #[expect(clippy::use_debug, reason = "Print the struct on --verbose")]
    pub fn format_debug(&self) -> String {
        format!("{:#?}", &self)
    }

    pub fn format_short(&self) -> String {
        format!("- {} {}", self.name, self.format_installed_version())
    }

    #[expect(dead_code, reason = "Useful variant of `vec_injected`")]
    pub fn vec_extras(&self) -> Vec<&str> {
        self.extras.iter().map(AsRef::as_ref).collect()
    }

    pub fn vec_injected(&self) -> Vec<&str> {
        self.injected.iter().map(AsRef::as_ref).collect()
    }

    pub fn format_extras(&self) -> String {
        self.extras
            .iter()
            .map(|extra| format!("'{}'", extra.green()))
            .join(",")
    }

    pub fn format_injected(&self) -> String {
        self.injected
            .iter()
            .map(|dep| format!("'{}'", dep.green()))
            .join(", ")
    }

    pub fn format_human(&self) -> anyhow::Result<String> {
        let mut result = format!("- {}", self.name);

        if !self.extras.is_empty() {
            // result.push_str(&format!("[{}]", self.format_extras()));
            write!(result, "[{}]", self.format_extras())?;
        }

        if !self.requested_version.is_empty() {
            // result.push_str(&format!(" {}", self.requested_version.cyan()));
            write!(result, " {}", self.requested_version.cyan())?;
        }

        if self.editable {
            // result.push_str(&format!(" {}", "--editable".yellow()));
            write!(result, " {}", "--editable".yellow())?;
        }

        result.push('\n');

        // result.push_str(&format!(
        //     "{}Installed Version: {} on {}.\n",
        //     INDENT,
        //     self.format_installed_version(),
        //     self.python.bright_blue()
        // ));
        writeln!(
            result,
            "{}Installed Version: {} on {}.",
            INDENT,
            self.format_installed_version(),
            self.python.bright_blue()
        )?;

        if self.outdated && !self.available_version.is_empty() {
            // result.push_str(&format!(
            //     "{}Available Version: {}.\n",
            //     INDENT,
            //     self.available_version.green(),
            // ));

            writeln!(
                result,
                "{}Available Version: {}.",
                INDENT,
                self.available_version.green(),
            )?;
        }

        if !self.injected.is_empty() {
            let formatted_injects = self.format_injected();
            // result.push_str(&format!("{INDENT}Injected Packages: {formatted_injects}\n"));
            writeln!(result, "{INDENT}Injected Packages: {formatted_injects}")?;
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
        // result.push_str(&format!("{INDENT}Scripts: {formatted_scripts}"));
        writeln!(result, "{INDENT}Scripts: {formatted_scripts}")?;

        Ok(result)
    }
}

/// Drop the `MAGIC_HEADER` from a buffer (if present)
pub fn strip_header(buf: &mut Vec<u8>) {
    // postponed: the current header is 7 chars long.
    //            the version should be ignored for starts_with,
    //            and if the length should change, this must be 'match'ed based on the version
    if buf.starts_with(MAGIC_HEADER_V2) {
        let _ = buf.drain(0..MAGIC_HEADER_V2.len());
    }
}

/// Prepend the `MAGIC_HEADER` to a buffer
pub fn add_header(buf: &mut Vec<u8>) {
    let mut new_buf = Vec::with_capacity(MAGIC_HEADER_V2.len() + buf.len());
    new_buf.extend_from_slice(MAGIC_HEADER_V2);
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
pub async fn load_generic_msgpack<'buf, T: serde::Deserialize<'buf>>(
    filename: &Path,
    buf: &'buf mut Vec<u8>,
) -> anyhow::Result<T> {
    // Open the msgpack file
    let mut file = File::open(filename).await?;

    // for some reason (I guess the lifetime), buf can just be passed around now?
    file.read_to_end(buf).await?;
    strip_header(buf);

    // Read the contents of the file into a Metadata struct
    let metadata: T = rmp_serde::decode::from_slice(buf)?;

    Ok(metadata)
}

#[expect(clippy::struct_excessive_bools, reason = "...")]
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct LoadMetadataConfig {
    pub recheck_scripts: bool,
    pub updates_check: bool,
    pub updates_prereleases: bool,
    pub updates_ignore_constraints: bool,
}

impl LoadMetadataConfig {
    #[expect(dead_code, reason = "Useful variant in addition to `none()`")]
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
}

impl Default for LoadMetadataConfig {
    fn default() -> Self {
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

pub async fn serialize_msgpack<T: serde::Serialize>(metadata: &T) -> anyhow::Result<Vec<u8>> {
    // Read the contents of the file into a Metadata struct
    let mut bytes = rmp_serde::encode::to_vec(metadata)?;

    add_header(&mut bytes);

    Ok(bytes)
}

/// Atomically writes the given bytes to the specified file.
///
/// This function first writes to `<filename>.tmp`, then renames it to `filename`.
/// This approach prevents data loss from incomplete writes, as the original file is
/// not modified until the entire write succeeds.
///
/// If any step fails, the temporary file is removed to avoid clutter.
pub async fn atomic_write<P: AsRef<Path>>(
    filename: P,
    bytes: &[u8],
) -> anyhow::Result<()> {
    let tmp_filename = filename.as_ref().with_extension("tmp");

    // Attempt to write and rename, cleaning up the temp file on error
    let result = async {
        let mut file = File::create(&tmp_filename).await?;
        file.write_all(bytes).await?;
        file.flush().await?;

        tokio::fs::rename(&tmp_filename, filename).await?;

        Ok(())
    }
    .await;

    // Clean up tmp file if something went wrong
    if result.is_err() {
        let _ = tokio::fs::remove_file(&tmp_filename).await;
    }

    result
}

pub async fn store_generic_msgpack<T: serde::Serialize>(
    filename: &Path,
    metadata: &T,
) -> anyhow::Result<()> {
    let bytes = serialize_msgpack(metadata).await?;
    atomic_write(filename, &bytes).await
}

pub async fn store_metadata(
    filename: &Path,
    metadata: &Metadata,
) -> anyhow::Result<()> {
    store_generic_msgpack(filename, metadata).await
}
