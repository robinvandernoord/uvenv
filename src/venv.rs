use crate::helpers::PathToString;
use crate::metadata::venv_path;
use crate::pip::parse_requirement;
use crate::uv::{uv, uv_venv};
use anyhow::{bail, Context};
use owo_colors::OwoColorize;
use std::env;
use std::path::{Path, PathBuf};
use uv_pep508::{PackageName, Requirement};

use uv_python::PythonEnvironment;

/// Create a new virtualenv via `uv venv` at a Path
pub async fn create_venv_raw(
    venv_path: &Path,
    python: Option<&String>,
    force: bool,
    with_pip: bool,
) -> anyhow::Result<()> {
    if !force && venv_path.exists() {
        bail!("'{}' is already installed.\nUse '{}' to update existing tools or pass '{}' to this command to ignore this message.",
                    &venv_path.to_str().unwrap_or_default().green(), "uvenv upgrade".green(), "--force".green())
    }

    let mut args: Vec<&str> = vec!["venv", venv_path.to_str().unwrap_or_default()];

    if let Some(py) = python {
        args.push("--python");
        args.push(py);
    }
    if with_pip {
        args.push("--seed");
    }

    uv(&args).await?;

    Ok(())
}

/// Create a new virtualenv from a parsed `PackageName`.
pub async fn create_venv(
    package_name: &PackageName,
    python: Option<&String>,
    force: bool,
    with_pip: bool,
    custom_prefix: Option<String>,
) -> anyhow::Result<PathBuf> {
    let venv_path = custom_prefix.map_or_else(
        || venv_path(package_name.as_ref()),
        |prefix| PathBuf::from(format!("{prefix}{package_name}")),
    );

    create_venv_raw(&venv_path, python, force, with_pip).await?;

    Ok(venv_path)
}

/// activate a venv (from Path) by setting the `VIRTUAL_ENV` and loading the `PythonEnvironment`.
pub async fn activate_venv(venv: &Path) -> anyhow::Result<PythonEnvironment> {
    let venv_str = venv.to_str().unwrap_or_default();
    env::set_var("VIRTUAL_ENV", venv_str);

    uv_venv(None).with_context(|| format!("Could not properly activate venv '{venv_str}'!"))
}

/// Find the path to an existing venv for an install spec str.
#[expect(
    dead_code,
    reason = "It can be useful to find a venv for an install spec later."
)]
pub async fn find_venv(install_spec: &str) -> Option<PathBuf> {
    let (requirement, _) = parse_requirement(install_spec).await.ok()?;
    let requirement_name = requirement.name.to_string();

    Some(venv_path(&requirement_name))
}

/// Parse an install spec str into a Requirement and create a new environment for it.
pub async fn setup_environ_from_requirement(
    install_spec: &str
) -> anyhow::Result<(Requirement, PythonEnvironment)> {
    let (requirement, _) = parse_requirement(install_spec).await?;
    let requirement_name = requirement.name.to_string();
    let venv_dir = venv_path(&requirement_name);
    if !venv_dir.exists() {
        bail!("No virtualenv for '{}'.", install_spec.green(),);
    }
    let environ = activate_venv(&venv_dir).await?;
    Ok((requirement, environ))
}

/// remove a venv directory
pub async fn remove_venv(venv: &PathBuf) -> anyhow::Result<()> {
    Ok(
        // ? + Ok for anyhow casting
        tokio::fs::remove_dir_all(venv).await?,
    )
}

/// Get the absolute path to a script in a venv.
pub fn venv_script(
    venv: &PythonEnvironment,
    script: &str,
) -> String {
    let script_path = venv.scripts().join(script);
    script_path.to_string()
}
