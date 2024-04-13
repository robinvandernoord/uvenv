use directories::ProjectDirs;
use owo_colors::OwoColorize;
use pep508_rs::{PackageName, Requirement};

use std::path::PathBuf;
use std::process::Command;

use uv_cache::Cache;
use uv_installer::SitePackages;
use uv_interpreter::PythonEnvironment;

pub fn uv(args: Vec<&str>) -> Result<bool, String> {
    let err_prefix = format!("uv {}", &args[0]);

    let result = Command::new("uv").args(args).output();

    match result {
        Ok(result) => match result.status.code() {
            Some(0) => Ok(true),
            Some(_) | None => {
                let err = String::from_utf8(result.stderr).unwrap_or_default();
                Err(format!("{} | {}", err_prefix, err))
            }
        },
        Err(result) => Err(format!("{} | {}", err_prefix, result.to_string())),
    }
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
        }
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

pub trait ExtractVersion {
    fn version(&self) -> String;
}

impl ExtractVersion for Requirement {
    fn version(&self) -> String {
        match self.version_or_url.clone() {
            Some(version) => match version {
                pep508_rs::VersionOrUrl::VersionSpecifier(v) => v.to_string(),
                _ => String::new(),
            },
            None => String::new(),
        }
    }
}
