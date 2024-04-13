use crate::cli::{InstallOptions, Process};
use crate::helpers::ResultToString;
use crate::metadata::{get_venv_dir, Metadata};
use crate::uv::{uv, uv_get_installed_version, uv_venv, ExtractVersion, Helpers};
use owo_colors::OwoColorize;
use pep508_rs::{PackageName, Requirement};
use std::path::{Path, PathBuf};
use std::str::FromStr;
use std::{env, fs};
use uv_interpreter::PythonEnvironment;

pub fn create_venv(
    package_name: &PackageName,
    python: Option<String>,
    force: bool,
    with_pip: bool,
) -> Result<PathBuf, String> {
    let venvs = get_venv_dir();
    let venv_path = venvs.join(&package_name.to_string());

    if !force && venv_path.exists() {
        return Err(
            format!("'{}' is already installed.\nUse '{}' to update existing tools or pass '{}' to this command to ignore this message.", 
            &package_name.to_string().green(), "uvx upgrade".green(), "--force".green())
        );
    }

    let mut args: Vec<&str> = vec!["venv", venv_path.to_str().unwrap_or_default()];

    // extract because 'if let Some()' has a too short lifetime.
    let py = python.unwrap_or_default();

    // if let Some(py) = python {
    if py != "" {
        args.push("--python");
        args.push(&py);
    }
    if with_pip {
        args.push("--seed");
    }

    uv(args)?;

    return Ok(venv_path);
}

pub fn activate_venv(venv: &PathBuf) -> Result<PythonEnvironment, String> {
    let venv_str = venv.to_str().unwrap_or_default();
    env::set_var("VIRTUAL_ENV", venv_str);

    return uv_venv(None)
        .ok_or_else(|| format!("Could not properly activate venv '{}'!", venv_str));
}

pub fn remove_venv(venv: &PathBuf) {
    let _ = fs::remove_dir_all(venv);
}
