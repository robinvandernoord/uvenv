use anyhow::{Context, bail};
use owo_colors::OwoColorize;

use crate::commands::create::create;
use crate::metadata::LoadMetadataConfig;
use crate::{
    cli::{Process, ReinstallOptions},
    commands::{install::install_package, uninstall::uninstall_package},
    metadata::{Metadata, venv_path},
    pip::parse_requirement,
    uv::ExtractInfo,
};

pub async fn reinstall(
    install_spec: &str,
    python: Option<&str>,
    force: bool,
    with_injected: bool,
    no_cache: bool,
    editable: bool,
) -> anyhow::Result<String> {
    let (requirement, _resolved_install_spec) = parse_requirement(install_spec).await?;
    let requirement_name = requirement.name.to_string();

    let venv_dir = venv_path(&requirement_name);

    if !venv_dir.exists() && !force {
        bail!(
            "'{requirement_name}' was not previously installed. Please run 'uvenv install {install_spec}' or pass `--force` instead.",
        );
    }

    let current_metadata =
        Metadata::for_requirement(&requirement, &LoadMetadataConfig::none()).await;

    let install_spec_changed =
        editable || !requirement.version().is_empty() || !requirement.extras().is_empty();

    if let Err(err) = uninstall_package(&requirement_name, force).await {
        eprintln!(
            "{}: something went wrong during uninstall ({});",
            "Warning".yellow(),
            err
        );
    }

    let new_install_spec = if install_spec_changed {
        install_spec
    } else {
        &current_metadata.install_spec
    };

    // use --python if specified, otherwise use the current python spec (which can be none)
    let python_spec = python.or(current_metadata.python_spec.as_deref());

    let inject = if with_injected {
        current_metadata.vec_injected()
    } else {
        Vec::new()
    };

    if new_install_spec.is_empty() {
        create(
            &current_metadata.name,
            python_spec,
            true, // force seed for now
            force,
        )
        .await
    } else {
        install_package(
            new_install_spec,
            None,
            python_spec,
            force,
            &inject,
            no_cache,
            editable,
        )
        .await
    }
}

impl Process for ReinstallOptions {
    async fn process(self) -> anyhow::Result<i32> {
        match reinstall(
            &self.package,
            self.python.as_deref(),
            self.force,
            !self.without_injected,
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
                    "Something went wrong trying to reinstall '{}';",
                    self.package
                )
            }),
        }
    }
}
