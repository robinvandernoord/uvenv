use crate::animate::{show_loading_indicator, AnimationSettings};
use crate::cli::{InstallOptions, Process};

use crate::metadata::Metadata;
use crate::pip::parse_requirement;
use crate::symlinks::{create_symlink, find_symlinks};
use crate::uv::{uv, uv_get_installed_version, ExtractInfo, Helpers};
use crate::venv::{activate_venv, create_venv, remove_venv};

use core::fmt::Display;
use owo_colors::OwoColorize;
use std::collections::BTreeMap;
use uv_pep508::Requirement;

use anyhow::{bail, Context};
use std::path::{Path, PathBuf};

use uv_python::PythonEnvironment;

pub async fn _install_package<S: AsRef<str>>(
    package_name: &str,
    inject: &[S],
    no_cache: bool,
    force: bool,
    editable: bool,
) -> anyhow::Result<bool> {
    let mut args: Vec<&str> = vec!["pip", "install"];

    if !inject.is_empty() {
        args.append(&mut inject.iter().map(AsRef::as_ref).collect());
    }

    if no_cache || force {
        args.push("--no-cache");
    }

    if editable {
        // -e should go right before package name!
        args.push("--editable");
    }
    args.push(package_name);

    let promise = uv(&args);

    show_loading_indicator(
        promise,
        format!("installing {package_name}"),
        AnimationSettings::default(),
    )
    .await
}

async fn ensure_venv(
    maybe_venv: Option<&Path>,
    requirement: &Requirement,
    python: Option<&String>,
    force: bool,
) -> anyhow::Result<PathBuf> {
    match maybe_venv {
        Some(venv) => {
            let buf = venv.to_path_buf();
            if buf.exists() {
                Ok(buf)
            } else {
                bail!("Package could not be installed because supplied venv was misssing.")
            }
        },
        None => create_venv(&requirement.name, python, force, true, None).await,
    }
}

async fn store_metadata<S: Display>(
    requirement_name: &str,
    requirement: &Requirement,
    inject: &[S],
    editable: bool,
    install_spec: &str,
    venv: &PythonEnvironment,
) -> anyhow::Result<Metadata> {
    let mut metadata = Metadata::new(requirement_name);
    let _ = metadata.fill(Some(venv));

    let python_info = venv.interpreter().markers();

    metadata.editable = editable;
    metadata.install_spec = String::from(install_spec);
    metadata.requested_version = requirement.version();

    metadata.python = format!(
        "{} {}",
        python_info.platform_python_implementation(),
        python_info.python_full_version()
    );
    metadata.python_raw = venv.stdlib_as_string();

    metadata.extras = requirement.extras();
    metadata.injected = inject.iter().map(ToString::to_string).collect();

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
) -> anyhow::Result<()> {
    let venv_root = venv.root();

    let symlinks = find_symlinks(requirement, &meta.installed_version, venv).await;

    let mut results = BTreeMap::new();
    for symlink in symlinks {
        let result = create_symlink(&symlink, venv_root, force, binaries).await;

        let success = result.unwrap_or_else(|msg| {
            eprintln!("⚠️ {}", msg.yellow());
            false
        });

        results.insert(symlink, success);
    }

    meta.scripts = results;
    meta.save(venv_root).await?;

    Ok(())
}

pub async fn install_package<S: AsRef<str> + Display>(
    install_spec: &str,
    maybe_venv: Option<&Path>,
    python: Option<&String>,
    force: bool,
    inject: &[S],
    no_cache: bool,
    editable: bool,
) -> anyhow::Result<String> {
    let (requirement, resolved_install_spec) = parse_requirement(install_spec).await?;

    let venv_path = ensure_venv(maybe_venv, &requirement, python, force).await?;
    let uv_venv = activate_venv(&venv_path).await?;

    if let Err(err) = _install_package(install_spec, inject, no_cache, force, editable).await {
        let _ = remove_venv(&venv_path).await;

        return Err(err);
    }

    let requirement_name = requirement.name.to_string();
    let mut metadata = store_metadata(
        &requirement_name,
        &requirement,
        inject,
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
    async fn process(self) -> anyhow::Result<i32> {
        match install_package(
            &self.package_name,
            None,
            self.python.as_ref(),
            self.force,
            &self.with,
            self.no_cache,
            self.editable,
        )
        .await
        {
            Ok(msg) => {
                println!("{msg}");
                Ok(0)
            },
            Err(msg) => Err(msg).with_context(|| {
                format!(
                    "Something went wrong trying to install '{}';",
                    self.package_name
                )
            }),
        }
    }
}
