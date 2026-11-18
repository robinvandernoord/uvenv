//! `uvenv outdated`: which installed packages have a newer version on the
//! package index, and which could not be checked, for people and (with
//! `--json`) for programs such as update managers.

use core::str::FromStr;

use futures::future;
use owo_colors::OwoColorize;
use serde::Serialize;
use uv_normalize::PackageName;
use uv_pep440::{Version, VersionSpecifier};

use crate::cli::{OutdatedOptions, Process};
use crate::commands::list::list_packages;
use crate::commands::self_info::CURRENT_UVENV_VERSION;
use crate::metadata::{LoadMetadataConfig, Metadata};
use crate::pip::is_git_install_spec;
use crate::pypi::{LookupError, try_get_versions_for_packagename};

/// A package with a newer version than the one installed.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Outdated {
    pub name: String,
    pub installed_version: String,
    pub available_version: String,
    pub install_spec: String,
}

/// A package whose versions could not be looked up, such as without a network.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Failed {
    pub name: String,
    pub error: String,
}

/// A package that is not looked up: one from git, a local folder or an
/// editable install, or one the package index does not have.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Skipped {
    pub name: String,
    pub reason: String,
}

/// uvenv itself, which `uvenv self update` updates.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct SelfVersion {
    pub installed_version: String,
    /// Empty when the index could not be asked.
    pub available_version: String,
    pub outdated: bool,
    /// Why the index could not be asked, if it could not.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize)]
pub struct Report {
    /// None when uvenv is not updated by itself, as in a snap.
    #[serde(rename = "self")]
    pub uvenv: Option<SelfVersion>,
    pub outdated: Vec<Outdated>,
    pub failed: Vec<Failed>,
    pub skipped: Vec<Skipped>,
}

/// What looking up one package found.
enum Found {
    Outdated(Outdated),
    Current,
    Failed(Failed),
    Skipped(Skipped),
}

/// Why a package is not looked up on the index, if it is not.
pub fn skip_reason(
    install_spec: &str,
    editable: bool,
) -> Option<&'static str> {
    let spec = install_spec.trim();
    let source = spec
        .split_once(" @ ")
        .map_or(spec, |(_, source)| source.trim());

    if editable {
        Some("editable install")
    } else if is_git_install_spec(spec) {
        Some("installed from git")
    } else if source.starts_with(['.', '/', '~']) || source.starts_with("file:") {
        Some("installed from a local path")
    } else {
        None
    }
}

/// The newest version the index offers, as `uvenv upgrade` would pick it.
async fn latest_version(
    name: &str,
    prereleases: bool,
    constraint: Option<VersionSpecifier>,
) -> Result<Option<Version>, LookupError> {
    let package_name =
        PackageName::from_str(name).map_err(|err| LookupError::Failed(err.to_string()))?;
    let versions =
        try_get_versions_for_packagename(&package_name, !prereleases, constraint).await?;
    Ok(versions.last().cloned())
}

async fn check_package(
    metadata: Metadata,
    prereleases: bool,
    ignore_constraints: bool,
) -> Found {
    if let Some(reason) = skip_reason(&metadata.install_spec, metadata.editable) {
        return Found::Skipped(Skipped {
            name: metadata.name,
            reason: reason.to_owned(),
        });
    }

    let constraint = if ignore_constraints || metadata.requested_version.is_empty() {
        None
    } else {
        VersionSpecifier::from_str(&metadata.requested_version).ok()
    };

    match latest_version(&metadata.name, prereleases, constraint).await {
        Err(LookupError::NotFound) => Found::Skipped(Skipped {
            name: metadata.name,
            reason: LookupError::NotFound.to_string(),
        }),
        Err(err) => Found::Failed(Failed {
            name: metadata.name,
            error: err.to_string(),
        }),
        Ok(None) => Found::Current,
        Ok(Some(latest)) => {
            let installed = metadata
                .find_currently_installed_version()
                .unwrap_or_else(|| metadata.installed_version_parsed());

            if latest > installed {
                Found::Outdated(Outdated {
                    installed_version: installed.to_string(),
                    available_version: latest.to_string(),
                    install_spec: metadata.install_spec,
                    name: metadata.name,
                })
            } else {
                Found::Current
            }
        },
    }
}

async fn check_self() -> Option<SelfVersion> {
    if cfg!(feature = "snap") {
        // updates managed by snapcraft
        return None;
    }

    let installed_version = CURRENT_UVENV_VERSION.to_owned();
    Some(match latest_version("uvenv", false, None).await {
        Ok(found) => {
            let outdated = found.as_ref().is_some_and(|latest| {
                Version::from_str(CURRENT_UVENV_VERSION).is_ok_and(|current| *latest > current)
            });
            SelfVersion {
                installed_version,
                available_version: found.as_ref().map_or_else(String::new, ToString::to_string),
                outdated,
                error: None,
            }
        },
        Err(err) => SelfVersion {
            installed_version,
            available_version: String::new(),
            outdated: false,
            error: Some(err.to_string()),
        },
    })
}

pub async fn outdated(
    venv_names: &[String],
    prereleases: bool,
    ignore_constraints: bool,
) -> anyhow::Result<Report> {
    let config = LoadMetadataConfig::none();
    let packages = list_packages(&config, Some(venv_names), None).await?;

    let checks = packages
        .into_iter()
        // uvenv's own environment, from install.sh, is reported as `self`.
        .filter(|metadata| metadata.name != "uvenv")
        .map(|metadata| check_package(metadata, prereleases, ignore_constraints));

    let (found, uvenv) = future::join(future::join_all(checks), check_self()).await;

    let mut report = Report {
        uvenv,
        ..Report::default()
    };

    for item in found {
        match item {
            Found::Outdated(outdated) => report.outdated.push(outdated),
            Found::Failed(failed) => report.failed.push(failed),
            Found::Skipped(skipped) => report.skipped.push(skipped),
            Found::Current => {},
        }
    }

    Ok(report)
}

impl Report {
    fn print_human(&self) {
        if let Some(uvenv) = &self.uvenv
            && uvenv.outdated
        {
            println!(
                "🔶 uvenv {} -> {} (use {})",
                uvenv.installed_version.red(),
                uvenv.available_version.green(),
                "uvenv self update".blue()
            );
        }

        if self.outdated.is_empty() {
            println!("{}", "✅ All packages are up to date.".green());
        }

        for outdated in &self.outdated {
            println!(
                "🔶 {} {} -> {}",
                outdated.name.bold(),
                outdated.installed_version.red(),
                outdated.available_version.green()
            );
        }

        for failed in &self.failed {
            eprintln!(
                "⚠️ {}: could not check for updates: {}",
                failed.name.bold(),
                failed.error.yellow()
            );
        }

        if !self.outdated.is_empty() {
            println!(
                "{}",
                "💡 Tip: you can use `uvenv upgrade <package>` to update a specific environment."
                    .blue()
            );
        }
    }
}

impl Process for OutdatedOptions {
    async fn process(self) -> anyhow::Result<i32> {
        let report = outdated(
            &self.venv_names,
            self.show_prereleases,
            self.ignore_constraints,
        )
        .await?;

        if self.json {
            println!("{}", serde_json::to_string_pretty(&report)?);
        } else {
            report.print_human();
        }

        Ok(0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn packages_without_an_index_version_are_skipped() {
        assert_eq!(skip_reason("ruff", false), None);
        assert_eq!(skip_reason("ruff<1", false), None);
        assert_eq!(skip_reason("ruff[all]", false), None);
        assert_eq!(skip_reason("ruff", true), Some("editable install"));
        assert_eq!(
            skip_reason(
                "tool @ git+https://github.com/owner/tool@0123456789abcdef",
                false
            ),
            Some("installed from git")
        );
        assert_eq!(
            skip_reason("./tool", false),
            Some("installed from a local path")
        );
        assert_eq!(
            skip_reason("/home/me/tool", false),
            Some("installed from a local path")
        );
        assert_eq!(
            skip_reason("tool @ file:///home/me/tool", false),
            Some("installed from a local path")
        );
    }

    #[test]
    fn the_report_has_a_stable_shape() {
        let report = Report {
            uvenv: Some(SelfVersion {
                installed_version: "3.12.0".into(),
                available_version: "3.13.0".into(),
                outdated: true,
                error: None,
            }),
            outdated: vec![Outdated {
                name: "ruff".into(),
                installed_version: "0.1.0".into(),
                available_version: "0.2.0".into(),
                install_spec: "ruff".into(),
            }],
            failed: vec![Failed {
                name: "b2".into(),
                error: "offline".into(),
            }],
            skipped: vec![Skipped {
                name: "tool".into(),
                reason: "installed from git".into(),
            }],
        };
        let json = serde_json::to_value(&report).expect("serializes");
        let text = |pointer: &str| json.pointer(pointer).and_then(serde_json::Value::as_str);
        assert_eq!(text("/self/available_version"), Some("3.13.0"));
        assert_eq!(json.pointer("/self/error"), None);
        assert_eq!(text("/outdated/0/name"), Some("ruff"));
        assert_eq!(text("/failed/0/error"), Some("offline"));
        assert_eq!(text("/skipped/0/reason"), Some("installed from git"));
    }
}
