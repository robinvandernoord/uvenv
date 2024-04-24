use itertools::Itertools;
use owo_colors::OwoColorize;
use pep508_rs::Requirement;
use uv_interpreter::PythonEnvironment;

use crate::helpers::StringExt;
use crate::venv::setup_environ_from_requirement;
use crate::{
    animate::{show_loading_indicator, AnimationSettings},
    cli::{Process, UpgradeOptions},
    metadata::Metadata,
    uv::{uv, uv_get_installed_version, ExtractInfo, Helpers},
};

pub async fn update_metadata(
    metadata: &mut Metadata,
    requirement: &Requirement,
    environ: &PythonEnvironment,
    requested_version: String,
) -> Result<String, String> {
    let new_version = uv_get_installed_version(&requirement.name, Some(environ))?;

    metadata.requested_version = requested_version;
    metadata.installed_version = new_version.clone();
    metadata.save(&environ.to_path_buf()).await?;

    Ok(new_version)
}

fn build_msg(
    old_version: String,
    new_version: String,
    metadata: &mut Metadata,
) -> Result<String, String> {
    let mut msg = String::new();
    if old_version == new_version {
        msg.push_str(&format!(
            "🌟 '{}' is already up to date at version {}!",
            &metadata.name.green(),
            &new_version.cyan()
        ));
        if !metadata.requested_version.is_empty() {
            msg.push_str(&format!("\n💡 This package was installed with a version constraint ({}). If you want to ignore this constraint, use `{}`.",
            &metadata.requested_version.cyan(),

            format!("uvx upgrade --force {}", &metadata.name).green()
        ));
        }
    } else {
        msg.push_str(&format!(
            "🚀 Successfully updated '{}' from version {} to version {}!",
            metadata.name.green(),
            old_version.cyan(),
            new_version.cyan()
        ));
    }

    Ok(msg)
}

pub async fn _upgrade_package(
    requirement: &Requirement,
    metadata: &mut Metadata,
    environ: &PythonEnvironment,
    force: bool,
    no_cache: bool,
    skip_injected: bool,
) -> Result<String, String> {
    let old_version = metadata.installed_version.clone();

    let mut args = vec!["pip", "install", "--upgrade"];

    if force || no_cache {
        args.push("--no-cache")
    }

    let version = requirement.version().or(if force {
        ""
    } else {
        &metadata.requested_version
    });

    let mut upgrade_spec = metadata.name.clone();

    let mut extras = metadata.extras.clone();
    extras.extend(requirement.extras());

    if !extras.is_empty() {
        upgrade_spec.push_str(&format!("[{}]", extras.iter().join(",")))
    }

    if !version.is_empty() {
        upgrade_spec.push_str(&version)
    }

    args.push(&upgrade_spec);

    if !skip_injected {
        args.extend(metadata.vec_injected());
    }

    let promise = uv(args);

    show_loading_indicator(
        promise,
        format!("upgrading {}", &metadata.name),
        AnimationSettings::default(),
    )
    .await?;

    let new_version = update_metadata(metadata, requirement, environ, version).await?;

    build_msg(old_version, new_version, metadata)
}

pub async fn upgrade_package(
    install_spec: &str,
    force: bool,
    no_cache: bool,
    skip_injected: bool,
) -> Result<String, String> {
    // No virtualenv for '{package_name}', stopping. Use 'uvx install' instead.
    let (requirement, environ) = setup_environ_from_requirement(install_spec).await?;

    let mut metadata = Metadata::for_requirement(&requirement, true).await;

    _upgrade_package(
        &requirement,
        &mut metadata,
        &environ,
        force,
        no_cache,
        skip_injected,
    )
    .await
}

impl Process for UpgradeOptions {
    async fn process(self) -> Result<i32, String> {
        match upgrade_package(
            &self.package_name,
            self.force,
            self.no_cache,
            self.skip_injected,
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
