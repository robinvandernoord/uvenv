use crate::animate::{show_loading_indicator, AnimationSettings};
use crate::cli::{InstallOptions, Process};

use crate::metadata::Metadata;
use crate::pip::parse_requirement;
use crate::symlinks::{create_symlink, find_symlinks};
use crate::uv::{uv, uv_get_installed_version, ExtractInfo, Helpers};
use crate::venv::{activate_venv, create_venv, remove_venv};

use owo_colors::OwoColorize;
use pep508_rs::Requirement;
use std::collections::HashMap;

use std::path::{Path, PathBuf};

use uv_interpreter::PythonEnvironment;

pub async fn _install_package(
    package_name: &str,
    inject: &[&str],
    no_cache: bool,
    force: bool,
    editable: bool,
) -> Result<bool, String> {
    let mut args: Vec<&str> = vec!["pip", "install"];

    if !inject.is_empty() {
        args.append(&mut inject.to_owned())
    }

    if no_cache || force {
        args.push("--no-cache")
    }

    if editable {
        // -e should go right before package name!
        args.push("--editable")
    }
    args.push(package_name);

    let promise = uv(args);

    show_loading_indicator(
        promise,
        format!("installing {}", package_name),
        AnimationSettings::default(),
    )
    .await
}

async fn ensure_venv(
    maybe_venv: Option<&Path>,
    requirement: &Requirement,
    python: Option<&String>,
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
        },
        None => create_venv(&requirement.name, python, force, true, None).await,
    }
}

async fn store_metadata(
    requirement_name: &str,
    requirement: &Requirement,
    inject: &[&str],
    editable: bool,
    install_spec: &str,
    venv: &PythonEnvironment,
) -> Result<Metadata, String> {
    let mut metadata = Metadata::new(requirement_name);
    let _ = metadata.fill(Some(venv));

    let python_info = venv.interpreter().markers();

    metadata.editable = editable;
    metadata.install_spec = String::from(install_spec);
    metadata.requested_version = requirement.version();

    metadata.python = format!(
        "{} {}",
        python_info.platform_python_implementation, python_info.python_full_version
    );
    metadata.python_raw = venv.stdlib_as_string();

    metadata.extras = requirement.extras();
    metadata.injected = inject.iter().map(|inj| inj.to_string()).collect();

    if let Ok(version) = uv_get_installed_version(&requirement.name, Some(venv)) {
        metadata.installed_version = version;
    }

    metadata.save(&venv.to_path_buf()).await?;
    Ok(metadata)
}

pub async fn install_symlinks(
    meta: &mut Metadata,
    venv: &PythonEnvironment,
    requirement: &Requirement,
    force: bool,
    binaries: &[&str],
) -> Result<(), String> {
    let venv_root = venv.root();

    let symlinks = find_symlinks(requirement, &meta.installed_version, venv).await;

    let mut results = HashMap::new();
    for symlink in symlinks {
        results.insert(
            symlink.clone(),
            create_symlink(&symlink, venv_root, force, binaries)
                .await
                .unwrap_or(false),
        );
    }

    meta.scripts = results;
    meta.save(venv_root).await?;

    Ok(())
}

pub async fn install_package(
    install_spec: &str,
    maybe_venv: Option<&Path>,
    python: Option<&String>,
    force: bool,
    inject: Vec<&str>,
    no_cache: bool,
    editable: bool,
) -> Result<String, String> {
    let (requirement, resolved_install_spec) = parse_requirement(install_spec).await?;

    let venv_path = ensure_venv(maybe_venv, &requirement, python, force).await?;
    let uv_venv = activate_venv(&venv_path).await?;

    if let Err(e) = _install_package(install_spec, &inject, no_cache, force, editable).await {
        let _ = remove_venv(&venv_path).await;

        return Err(e);
    }

    let requirement_name = requirement.name.to_string();
    let mut metadata = store_metadata(
        &requirement_name,
        &requirement,
        &inject,
        editable,
        &resolved_install_spec,
        &uv_venv,
    )
    .await?;

    install_symlinks(&mut metadata, &uv_venv, &requirement, force, &[]).await?;

    Ok(format!(
        "📦 {} ({}) installed!",
        requirement_name,
        metadata.installed_version.cyan()
    )) // :package:
}

impl Process for InstallOptions {
    async fn process(self) -> Result<i32, String> {
        match install_package(
            &self.package_name,
            None,
            self.python.as_ref(),
            self.force,
            vec![],
            self.no_cache,
            self.editable,
        )
        .await
        {
            Ok(msg) => {
                println!("{}", msg);
                Ok(0)
            },
            Err(msg) => Err(msg),
        }
    }
}
