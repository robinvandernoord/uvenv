use anyhow::bail;
use core::fmt::Write;
use itertools::Itertools;
use owo_colors::OwoColorize;
use uv_pep508::Requirement;
use uv_python_interpreter::PythonEnvironment;

use crate::commands::list::{is_uvenv_outdated, list_packages};
use crate::commands::upgrade_all::upgrade_all;
use crate::helpers::StringExt;
use crate::metadata::LoadMetadataConfig;
use crate::pip::is_git_install_spec;
use crate::venv::setup_environ_from_requirement;
use crate::{
    animate::{AnimationSettings, show_loading_indicator},
    cli::{Process, UpgradeOptions},
    metadata::Metadata,
    uv::{ExtractInfo, Helpers, uv, uv_get_installed_version},
};

pub async fn update_metadata(
    metadata: &mut Metadata,
    requirement: &Requirement,
    environ: &PythonEnvironment,
    requested_version: String,
) -> anyhow::Result<String> {
    let new_version = uv_get_installed_version(&requirement.name, Some(environ))?;

    metadata.requested_version = requested_version;
    metadata.installed_version.clone_from(&new_version);
    metadata.save(&environ.to_path_buf()).await?;

    Ok(new_version)
}

fn build_msg(
    old_version: &str,
    new_version: &str,
    metadata: &Metadata,
) -> String {
    let mut msg = String::new();
    if old_version == new_version {
        // msg.push_str(&format!(
        //     "🌟 '{}' is already up to date at version {}!",
        //     &metadata.name.green(),
        //     &new_version.cyan()
        // ));
        let _ = write!(
            msg,
            "🌟 '{}' is already up to date at version {}!",
            metadata.name.green(),
            new_version.cyan()
        );

        if !metadata.requested_version.is_empty() {
            // msg.push_str(&format!("\n💡 This package was installed with a version constraint ({}). If you want to ignore this constraint, use `{}`.",
            //                       &metadata.requested_version.cyan(),
            //                       format!("uvenv upgrade --force {}", &metadata.name).green()
            // ));
            let _ = write!(
                msg,
                "\n💡 This package was installed with a version constraint ({}). If you want to ignore this constraint, use `{}`.",
                metadata.requested_version.cyan(),
                format!("uvenv upgrade --force {}", metadata.name).green()
            );
        }
    } else {
        // msg.push_str(&format!(
        //     "🚀 Successfully updated '{}' from version {} to version {}!",
        //     metadata.name.green(),
        //     old_version.cyan(),
        //     new_version.cyan()
        // ));
        let _ = write!(
            msg,
            "🚀 Successfully updated '{}' from version {} to version {}!",
            metadata.name.green(),
            old_version.cyan(),
            new_version.cyan()
        );
    }

    msg
}

pub async fn upgrade_package_from_requirement(
    requirement: &Requirement,
    metadata: &mut Metadata,
    environ: &PythonEnvironment,
    force: bool,
    no_cache: bool,
    skip_injected: bool,
) -> anyhow::Result<String> {
    let old_version = metadata.installed_version.clone();

    let mut args = vec!["pip", "install", "--upgrade"];

    if force || no_cache {
        args.push("--no-cache");
    }

    let is_git_source = is_git_install_spec(&metadata.install_spec);
    let version = if is_git_source {
        String::new()
    } else {
        requirement.version().or(if force {
            ""
        } else {
            &metadata.requested_version
        })
    };

    let mut extras = metadata.extras.clone();
    extras.extend(requirement.extras());

    let mut upgrade_spec = if is_git_source {
        if let Some((_, source)) = metadata.install_spec.split_once(" @ ") {
            let mut requirement_name = metadata.name.clone();
            if !extras.is_empty() {
                write!(requirement_name, "[{}]", extras.iter().join(","))?;
            }
            format!("{requirement_name} @ {source}")
        } else {
            metadata.install_spec.clone()
        }
    } else {
        let mut requirement_name = metadata.name.clone();
        if !extras.is_empty() {
            // upgrade_spec.push_str(&format!("[{}]", extras.iter().join(",")));
            write!(requirement_name, "[{}]", extras.iter().join(","))?;
        }
        requirement_name
    };

    if !version.is_empty() {
        upgrade_spec.push_str(&version);
    }

    args.push(&upgrade_spec);

    if !skip_injected {
        args.extend(metadata.vec_injected());
    }

    let promise = uv(&args);

    show_loading_indicator(
        promise,
        format!("upgrading {}", metadata.name),
        AnimationSettings::default(),
    )
    .await?;

    let new_version = update_metadata(metadata, requirement, environ, version).await?;

    Ok(build_msg(&old_version, &new_version, metadata))
}

pub async fn upgrade_package(
    install_spec: &str,
    force: bool,
    no_cache: bool,
    skip_injected: bool,
) -> anyhow::Result<String> {
    // No virtualenv for '{package_name}', stopping. Use 'uvenv install' instead.
    let (requirement, environ) = setup_environ_from_requirement(install_spec).await?;

    // = LoadMetadataConfig::default with one change:
    let config = LoadMetadataConfig {
        updates_check: false,
        ..Default::default()
    };

    let mut metadata = Metadata::for_requirement(&requirement, &config).await;

    upgrade_package_from_requirement(
        &requirement,
        &mut metadata,
        &environ,
        force,
        no_cache,
        skip_injected,
    )
    .await
}

async fn find_outdated() -> Vec<String> {
    let config = LoadMetadataConfig {
        recheck_scripts: false,
        updates_check: true,
        updates_prereleases: false,
        updates_ignore_constraints: false,
    };

    let packages_info = list_packages(&config, None, None).await.unwrap_or_default();

    packages_info
        .into_iter()
        .filter_map(|meta| {
            should_auto_upgrade(meta.outdated, &meta.install_spec).then_some(meta.name)
        })
        .collect()
}

fn should_auto_upgrade(
    outdated: bool,
    install_spec: &str,
) -> bool {
    outdated || is_git_install_spec(install_spec)
}

impl Process for UpgradeOptions {
    async fn process(self) -> anyhow::Result<i32> {
        let self_outdated = is_uvenv_outdated(true).await;

        let package_names = if self.package_names.is_empty() {
            let outdated = find_outdated().await;

            if self_outdated {
                eprintln!(
                    "{} Use {} to get the latest version.",
                    "A newer version of uvenv is available.".yellow(),
                    "uvenv self update".blue()
                );
            }

            if outdated.is_empty() {
                bail!("{}", "No packages are outdated.".blue());
            }

            outdated
        } else {
            self.package_names
        };

        upgrade_all(
            self.force,
            self.no_cache,
            self.skip_injected,
            &package_names,
        )
        .await
        .map(|()| 0)
    }
}

#[cfg(test)]
mod tests {
    use super::should_auto_upgrade;

    #[test]
    fn automatic_upgrade_includes_git_sources_even_without_a_pypi_update() {
        assert!(should_auto_upgrade(
            false,
            "package @ git+https://git.example.com/team/repo.git@main"
        ));
        assert!(should_auto_upgrade(true, "regular-package"));
        assert!(!should_auto_upgrade(false, "regular-package"));
    }
}
