use crate::animate::{show_loading_indicator, AnimationSettings};
use crate::cli::{InstallOptions, Process};
use crate::helpers::ResultToString;
use crate::metadata::Metadata;
use crate::symlinks::{create_symlink, find_symlinks};
use crate::uv::{uv, uv_get_installed_version, ExtractVersion, Helpers};
use crate::venv::{activate_venv, create_venv, remove_venv};

use pep508_rs::Requirement;
use std::collections::HashMap;

use std::path::{Path, PathBuf};
use std::str::FromStr;

use uv_interpreter::PythonEnvironment;

async fn _install_package(
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

    let promise = uv(args);

    return show_loading_indicator(
        promise,
        format!("installing {}", package_name),
        AnimationSettings::default(),
    )
    .await;
}

async fn ensure_venv(
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
        None => create_venv(&requirement.name, python, force, true).await,
    }
}

async fn store_metadata(
    requirement_name: &str,
    requirement: &Requirement,
    inject: &Vec<&str>,
    install_spec: &str,
    venv: &PythonEnvironment,
) -> Result<Metadata, String> {
    let mut metadata = Metadata::new(&requirement_name);
    let _ = metadata.fill(Some(&venv));

    let python_info = venv.interpreter().markers();

    metadata.install_spec = String::from(install_spec);

    metadata.requested_version = requirement.version();

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

    metadata.save(&venv.to_path_buf()).await?;
    return Ok(metadata);
}

pub async fn install_symlinks(
    meta: &mut Metadata,
    venv: &PythonEnvironment,
    force: bool,
    binaries: &[&str],
) -> Result<(), String> {
    let venv_root = venv.root();

    let symlinks = find_symlinks(&meta, venv).await;

    let mut results = HashMap::new();
    for symlink in symlinks {
        results.insert(
            symlink.clone(),
            create_symlink(&symlink, &venv_root, force, binaries)
                .await
                .unwrap_or(false),
        );
    }

    meta.scripts = results;
    meta.save(&venv_root.to_path_buf()).await?;

    Ok(())
}

pub async fn install_package(
    install_spec: &str,
    maybe_venv: Option<&Path>,
    python: Option<String>,
    force: bool,
    inject: Vec<&str>,
    no_cache: bool,
) -> Result<String, String> {
    let requirement = Requirement::from_str(install_spec).map_err_to_string()?;

    let venv_path = ensure_venv(maybe_venv, &requirement, python, force).await?;
    let uv_venv = activate_venv(&venv_path).await?;

    if let Err(e) = _install_package(install_spec, &inject, no_cache, force).await {
        let _ = remove_venv(&venv_path).await;

        return Err(e);
    }

    let requirement_name = requirement.name.to_string();
    let mut metadata = store_metadata(
        &requirement_name,
        &requirement,
        &inject,
        &install_spec,
        &uv_venv,
    )
    .await?;

    install_symlinks(&mut metadata, &uv_venv, force, &[]).await?;

    Ok(format!(
        "📦 {} ({}) installed!",
        requirement_name, metadata.installed_version
    )) // :package:
}

impl Process for InstallOptions {
    async fn process(self) -> Result<u32, String> {
        match install_package(
            &self.package_name,
            None,
            self.python,
            self.force,
            vec![],
            self.no_cache,
        )
        .await
        {
            Ok(msg) => {
                println!("{}", msg);
                return Ok(0);
            }
            Err(msg) => return Err(msg),
        }
    }
}
