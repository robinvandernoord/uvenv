use std::ffi::OsStr;
use std::{collections::HashSet, path::PathBuf};

use crate::cmd::{find_sibling, run, run_print_output};
use directories::ProjectDirs;
use owo_colors::OwoColorize;
use pep508_rs::{PackageName, Requirement};
use uv_cache::Cache;
use uv_installer::SitePackages;
use uv_interpreter::PythonEnvironment;

use crate::helpers::PathToString;

pub async fn _get_uv_binary() -> Option<String> {
    // if bundled with entrypoint:
    // arg 0 = python
    // arg 1 = .../bin/uvx
    // elif bundled as bin, use current_exe (because arg 0 is just 'uvx' instead of a path):

    // let Some(binary) = env::args().nth(0) else {
    //     return None;
    // };
    // let Ok(binary_path) = PathBuf::from_str(&binary) else {
    //     return None;
    // };

    find_sibling("uv").await.map(|buf| buf.to_string())
}

pub async fn get_uv_binary() -> String {
    _get_uv_binary().await.unwrap_or_else(
        // fallback, hope 'uv' is available in global scope:
        || String::from("uv"),
    )
}

pub async fn uv<S>(args: Vec<S>) -> Result<bool, String>
where
    S: AsRef<OsStr>,
{
    // venv could be unavailable, use 'uv' from this library's requirement
    let script = get_uv_binary().await;

    let subcommand = &args[0].as_ref().to_str().unwrap_or_default(); // cursed but makes it work with both &str and String
    let err_prefix = format!("uv {}", subcommand);

    run(&script, args, Some(err_prefix)).await
}

pub async fn uv_with_output<S: AsRef<OsStr>>(args: Vec<S>) -> Result<i32, String> {
    let script = get_uv_binary().await;
    run_print_output(script, args).await
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
        // for result in pkgs.get_packages(package_name) {
        if let Some(result) = pkgs.get_packages(package_name).into_iter().next() {
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
        stdlib.unwrap_or_default().to_string()
    }
}

pub trait ExtractInfo {
    fn version(&self) -> String;
    fn extras(&self) -> HashSet<String>;
}

impl ExtractInfo for Requirement {
    fn version(&self) -> String {
        match self.version_or_url.clone() {
            Some(pep508_rs::VersionOrUrl::VersionSpecifier(v)) => v.to_string(),
            _ => String::new(),
        }
    }

    fn extras(&self) -> HashSet<String> {
        self.extras.iter().map(|extra| extra.to_string()).collect()
    }
}
