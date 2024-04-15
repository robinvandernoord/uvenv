use directories::ProjectDirs;
use owo_colors::OwoColorize;
use pep508_rs::{PackageName, Requirement};
use std::env::{self};
use std::str::FromStr;
use std::{ffi::OsStr, process::Stdio};
use tokio::fs::canonicalize;

use std::{collections::HashSet, path::PathBuf};
use tokio::process::Command;

use uv_cache::Cache;
use uv_installer::SitePackages;
use uv_interpreter::PythonEnvironment;

use crate::helpers::{PathToString, ResultToString};

pub async fn _get_uv_binary() -> Option<String> {
    // if bundled with entrypoint:
    // arg 0 = python
    // arg 1 = .../bin/uvx (arg 0 otherwise)

    let Some(binary) = env::args().nth(0) else {
        return None;
    };

    let Ok(binary_path) = PathBuf::from_str(&binary) else {
        return None;
    };

    let Ok(real_path) = canonicalize(&binary_path).await else {
        return None;
    };

    let Some(parent) = real_path.parent() else {
        return None;
    };

    // resolve symlinks etc:

    let uv_binary = parent.join("uv").to_string();

    Some(uv_binary)
}

pub async fn get_uv_binary() -> String {
    match _get_uv_binary().await {
        Some(bin) => bin,
        None => String::from("uv"), // fallback, hope 'uv' is available in global scope
    }
}

pub async fn uv<S>(args: Vec<S>) -> Result<bool, String>
where
    S: AsRef<OsStr>,
{
    // venv could be unavailable, use 'uv' from this library's requirement
    let script = get_uv_binary().await;

    let subcommand = &args[0].as_ref().to_str().unwrap_or_default(); // cursed but makes it work with both &str and String
    let err_prefix = format!("uv {}", subcommand);

    let result = Command::new(script).args(args).output().await;

    match result {
        Ok(result) => match result.status.code() {
            Some(0) => Ok(true),
            Some(_) | None => {
                let err = String::from_utf8(result.stderr).unwrap_or_default();
                Err(format!("{} | {}", err_prefix, err))
            },
        },
        Err(result) => Err(format!("{} | {}", err_prefix, result.to_string())),
    }
}

pub async fn run_with_output<S1, S2>(
    command: S1,
    args: Vec<S2>,
) -> Result<(), String>
where
    S1: AsRef<OsStr>,
    S2: AsRef<OsStr>,
{
    let mut cmd = Command::new(command);
    cmd.args(args);
    cmd.stdout(Stdio::inherit());
    cmd.stderr(Stdio::inherit());
    cmd.status().await.map_err_to_string()?;

    Ok(())
}

pub async fn uv_with_output<S>(args: Vec<S>) -> Result<(), String>
where
    S: AsRef<OsStr>,
{
    let script = get_uv_binary().await;

    return run_with_output(script, args).await;
}

pub fn uv_cache() -> Option<Cache> {
    if let Some(project_dirs) = ProjectDirs::from("", "", "uv") {
        Cache::from_path(project_dirs.cache_dir())
    } else {
        Cache::from_path(".uv_cache")
    }
    .ok()
}

pub fn uv_venv(maybe_cache: Option<Cache>) -> Option<PythonEnvironment> {
    if let Some(cache) = maybe_cache.or_else(uv_cache) {
        PythonEnvironment::from_virtualenv(&cache).ok()
    } else {
        None
    }
}

pub fn uv_get_installed_version(
    package_name: &PackageName,
    maybe_venv: Option<&PythonEnvironment>,
) -> Result<String, String> {
    let _venv: PythonEnvironment; // lifetime for if maybe_venv is None

    let site_packages = match maybe_venv {
        Some(venv) => SitePackages::from_executable(venv),
        None => {
            _venv = uv_venv(None).ok_or_else(|| format!("{}", "Failed to set up venv!".red()))?;
            SitePackages::from_executable(&_venv)
        },
    }
    .ok();

    if let Some(pkgs) = site_packages {
        for result in pkgs.get_packages(package_name) {
            return Ok(result.version().to_string());
        }
    };

    Err(format!(
        "No version found for '{}'.",
        package_name.to_string().yellow()
    ))
}

pub trait Helpers {
    fn to_path_buf(&self) -> PathBuf;
    fn stdlib_as_string(&self) -> String;
}

impl Helpers for PythonEnvironment {
    fn to_path_buf(&self) -> PathBuf {
        return self.root().to_path_buf();
    }

    fn stdlib_as_string(&self) -> String {
        let stdlib = self.interpreter().stdlib().to_str();
        return format!("{}", stdlib.unwrap_or_default());
    }
}

pub trait ExtractInfo {
    fn version(&self) -> String;
    fn extras(&self) -> HashSet<String>;
}

impl ExtractInfo for Requirement {
    fn version(&self) -> String {
        match self.version_or_url.clone() {
            Some(version) => match version {
                pep508_rs::VersionOrUrl::VersionSpecifier(v) => v.to_string(),
                _ => String::new(),
            },
            None => String::new(),
        }
    }

    fn extras(&self) -> HashSet<String> {
        self.extras.iter().map(|extra| extra.to_string()).collect()
    }
}
