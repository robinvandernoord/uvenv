use crate::helpers::PathToString;
use crate::metadata::venv_path;
use crate::pip::parse_requirement;
use crate::uv::{uv, uv_venv};
use anyhow::{bail, Context};
use owo_colors::OwoColorize;
use pep508_rs::{PackageName, Requirement};
use std::path::{Path, PathBuf};

use uv_python::PythonEnvironment;

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

pub async fn activate_venv(venv: &Path) -> anyhow::Result<PythonEnvironment> {
    let venv_str = venv.to_str().unwrap_or_default();
    std::env::set_var("VIRTUAL_ENV", venv_str);

    uv_venv(None).with_context(|| format!("Could not properly activate venv '{venv_str}'!"))
}

#[allow(dead_code)]
pub async fn find_venv(install_spec: &str) -> Option<PathBuf> {
    let (requirement, _) = parse_requirement(install_spec).await.ok()?;
    let requirement_name = requirement.name.to_string();

    Some(venv_path(&requirement_name))
}

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

pub async fn remove_venv(venv: &PathBuf) -> anyhow::Result<()> {
    tokio::fs::remove_dir_all(venv).await?;
    Ok(())
}

pub fn venv_script(
    venv: &PythonEnvironment,
    script: &str,
) -> String {
    let script_path = venv.scripts().join(script);
    script_path.to_string()
}
