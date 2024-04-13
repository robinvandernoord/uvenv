use crate::cli::{InstallOptions, Process};
use crate::helpers::ResultToString;
use crate::metadata::{get_venv_dir, Metadata};
use crate::symlinks::{create_symlink, find_symlinks};
use crate::uv::{uv, uv_get_installed_version, uv_venv, ExtractVersion, Helpers};
use crate::venv::{activate_venv, create_venv, remove_venv};
use owo_colors::OwoColorize;
use pep508_rs::{PackageName, Requirement};
use std::collections::HashMap;
use std::fmt::format;
use std::path::{Path, PathBuf};
use std::str::FromStr;
use std::{env, fs};
use uv_interpreter::PythonEnvironment;

fn _install_package(
    package_name: &str,
    inject: &Vec<&str>,
    no_cache: bool,
    force: bool,
) -> Result<bool, String> {
    let mut args: Vec<&str> = vec!["pip", "install", package_name];

    if inject.len() > 0 {
        args.append(&mut inject.clone())
    }

    if no_cache || force {
        args.push("--no-cache")
    }

    return uv(args);
}

fn ensure_venv(
    maybe_venv: Option<&Path>,
    requirement: &Requirement,
    python: Option<String>,
    force: bool,
) -> Result<PathBuf, String> {
    match maybe_venv {
        Some(venv) => {
            let buf = venv.to_path_buf();
            if !buf.exists() {
                Err(String::from(
                    "Package could not be installed because supplied venv was misssing.",
                ))
            } else {
                Ok(buf)
            }
        }
        None => create_venv(&requirement.name, python, force, true),
    }
}

fn store_metadata(
    requirement_name: &str,
    requirement: &Requirement,
    inject: &Vec<&str>,
    install_spec: &str,
    venv: &PythonEnvironment,
) -> Result<Metadata, String> {
    let mut metadata = Metadata::new(&requirement_name);

    let python_info = venv.interpreter().markers();

    metadata.install_spec = String::from(install_spec);

    metadata.requested_version = requirement.version();
    metadata.installed_version = String::new();

    metadata.python = format!(
        "{} {}",
        python_info.platform_python_implementation, python_info.python_full_version
    );
    metadata.python_raw = venv.stdlib_as_string();

    metadata.extras = requirement
        .extras
        .iter()
        .map(|extra| extra.to_string())
        .collect();

    metadata.injected = inject.iter().map(|inj| inj.to_string()).collect();

    if let Ok(version) = uv_get_installed_version(&requirement.name, Some(venv)) {
        metadata.installed_version = version;
    }

    metadata.save(&venv.to_path_buf())?;
    return Ok(metadata);
}

pub fn install_symlinks(
    meta: &mut Metadata,
    venv: &PythonEnvironment,
    force: bool,
    binaries: &[&str],
) -> Result<(), String> {
    let venv_root = venv.root();

    let symlinks = find_symlinks(&meta, venv);

    let mut results = HashMap::new();
    for symlink in symlinks {
        results.insert(
            symlink.clone(),
            create_symlink(&symlink, &venv_root, force, binaries).unwrap_or(false),
        );
    }

    meta.scripts = results;
    meta.save(&venv_root.to_path_buf())?;

    Ok(())
}

pub fn install_package(
    install_spec: &str,
    maybe_venv: Option<&Path>,
    python: Option<String>,
    force: bool,
    inject: Vec<&str>,
    no_cache: bool,
) -> Result<String, String> {
    let requirement = Requirement::from_str(install_spec).map_err_to_string()?;

    let venv_path = ensure_venv(maybe_venv, &requirement, python, force)?;
    let uv_venv = activate_venv(&venv_path)?;

    if let Err(e) = _install_package(install_spec, &inject, no_cache, force) {
        remove_venv(&venv_path);

        return Err(e);
    }

    let requirement_name = requirement.name.to_string();
    let mut metadata = store_metadata(
        &requirement_name,
        &requirement,
        &inject,
        &install_spec,
        &uv_venv,
    )?;

    install_symlinks(&mut metadata, &uv_venv, force, &[])?;

    Ok(format!(
        "📦 {} ({}) installed!",
        requirement_name, metadata.installed_version
    )) // :package:
}

impl Process for InstallOptions {
    fn process(self) -> Result<u32, String> {
        match install_package(
            &self.package_name,
            None,
            self.python,
            self.force,
            vec![],
            self.no_cache,
        ) {
            Ok(msg) => {
                println!("{}", msg);
                return Ok(0);
            }
            Err(msg) => return Err(msg),
        }
    }
}
