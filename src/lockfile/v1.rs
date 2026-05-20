use crate::cli::{FreezeOptions, OutputFormat, ThawOptions};
use crate::commands::freeze::Freeze;
use crate::commands::install::install_package;
use crate::commands::list::list_packages;
use crate::commands::thaw::Thaw;
use crate::helpers::{PathAsStr, ResultToString};
use crate::lockfile::{AutoDeserialize, Lockfile, PackageMap, PackageSpec, extract_python_version};
use crate::metadata::{LoadMetadataConfig, Metadata, get_venv_dir, serialize_msgpack, venv_path};
use crate::venv::remove_venv;
use anyhow::{Context, anyhow};
use core::fmt::Debug;
use itertools::Itertools;
use owo_colors::OwoColorize;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Default, Serialize, Deserialize)]
pub struct LockfileV1 {
    version: i8,
    packages: PackageMap<PackageSpecV1>,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Default, Serialize, Deserialize)]
struct PackageSpecV1 {
    spec: String,
    #[serde(default)]
    version: String,
    #[serde(default)]
    python: Option<String>,
    #[serde(default)]
    injected: Vec<String>,
    #[serde(default)]
    editable: bool,
}

impl Lockfile<'_, PackageSpecV1> for LockfileV1 {
    fn new(packages: PackageMap<PackageSpecV1>) -> Self {
        Self {
            version: 1,
            packages,
        }
    }

    async fn serialize_and_patch(
        &self,
        options: &FreezeOptions,
    ) -> anyhow::Result<Vec<u8>> {
        let serialized = match options.format {
            OutputFormat::TOML => {
                // this `to_document` converts everything to inline tables:
                let mut doc = toml_edit::ser::to_document(self)?;

                // now convert all top-level tables from inline to regular:
                for (_, item) in doc.iter_mut() {
                    // Attempt to convert the inline table into a normal table.
                    // Here we use as_inline_table_mut; if the packages field is indeed an inline table,
                    // we can take it out and call .into_table() to convert it.
                    if let Some(inline_table) = item.as_inline_table_mut() {
                        // Replace the inline table with a block table.
                        // Note: std::mem::take clears the inline table, leaving an empty one behind.
                        let table = core::mem::take(inline_table).into_table();
                        *item = toml_edit::Item::Table(table);
                    }
                }

                doc.to_string().into_bytes()
            },
            OutputFormat::JSON => serde_json::to_string_pretty(self)?.into_bytes(),
            OutputFormat::Binary => serialize_msgpack(self).await?,
        };

        Ok(serialized)
    }
}

impl From<Metadata> for PackageSpecV1 {
    fn from(value: Metadata) -> Self {
        let version = if value.requested_version.is_empty() {
            format!("~={}", value.installed_version)
        } else {
            value.requested_version
        };

        let python = extract_python_version(&value.python);

        let injected = value.injected.into_iter().collect();

        Self {
            spec: value.install_spec,
            editable: value.editable,
            version,
            python,
            injected,
        }
    }
}

impl PackageSpec for PackageSpecV1 {}

impl Freeze for LockfileV1 {
    async fn freeze(options: &FreezeOptions) -> anyhow::Result<i32> {
        let pkg_metadata = list_packages(&LoadMetadataConfig::none(), None, None).await?;

        let packages: PackageMap<PackageSpecV1> = if !options.include.is_empty() {
            // --include passed
            pkg_metadata
                .into_iter()
                .filter_map(|meta| {
                    options
                        .include
                        .contains(&meta.name)
                        .then(|| (meta.name.clone(), meta.into()))
                })
                .collect()
        } else if !options.exclude.is_empty() {
            // --exclude passed
            pkg_metadata
                .into_iter()
                .filter_map(|meta| {
                    if options.exclude.contains(&meta.name) {
                        None
                    } else {
                        Some((meta.name.clone(), meta.into()))
                    }
                })
                .collect()
        } else {
            // just do all
            pkg_metadata
                .into_iter()
                .map(|meta| (meta.name.clone(), meta.into()))
                .collect()
        };

        Ok(Self::write(packages, options).await?.into())
    }
}

impl Thaw for LockfileV1 {
    async fn thaw(
        options: &ThawOptions,
        data: &[u8],
        format: OutputFormat,
    ) -> anyhow::Result<i32> {
        let instance = match Self::from_format(data, format) {
            Err(err) => return Err(err).with_context(|| "Could not thaw data."),
            Ok(instance) => instance,
        };

        let mut possible_errors: Vec<Result<(), String>> = Vec::new();

        if options.remove_current {
            let venvs_dir = get_venv_dir();
            possible_errors.push(
                tokio::fs::remove_dir_all(&venvs_dir)
                    .await
                    .with_context(|| {
                        format!("Trying to remove all venvs at {}", venvs_dir.as_str().red())
                    })
                    .map_err_to_string(),
            );
        }

        let to_install = if !options.include.is_empty() {
            instance
                .packages
                .into_iter()
                .filter(|(name, _)| options.include.contains(name))
                .collect()
        } else if !options.exclude.is_empty() {
            instance
                .packages
                .into_iter()
                .filter(|(name, _)| !options.exclude.contains(name))
                .collect()
        } else {
            instance.packages
        };

        for (name, pkg) in to_install {
            let python_lower = options.python.to_lowercase();
            let python: Option<&str> = match python_lower.as_ref() {
                "frozen" => {
                    // default: use the python version from the lockfile:
                    pkg.python.as_deref() // Option<String> -> Option<&str>
                },
                "ignore" => {
                    // use (system) default:
                    None
                },
                specific => {
                    // use specific one:
                    Some(specific)
                },
            };

            let venv_path = venv_path(&name);

            if venv_path.exists() {
                if options.skip_current {
                    continue;
                }
                possible_errors.push(
                    remove_venv(&venv_path)
                        .await
                        .with_context(|| {
                            format!("Trying to remove venv {}", venv_path.as_str().red())
                        })
                        .map_err_to_string(),
                );
            }

            let spec = if pkg.version.starts_with('~') & !pkg.spec.contains('@') {
                // soft versioned spec:
                format!("{}{}", pkg.spec, pkg.version)
            } else {
                // hard versioned spec:
                pkg.spec
            };

            possible_errors.push(
                install_package(
                    &spec,
                    None,
                    python,
                    true,
                    &pkg.injected,
                    false,
                    pkg.editable,
                )
                .await
                .map(|feedback| println!("{feedback}"))
                .with_context(|| format!("Trying to install {}", name.red()))
                .map_err_to_string(),
            );
        }

        let errors = possible_errors
            .into_iter()
            .filter_map(Result::err)
            .join("\n");

        if errors.is_empty() {
            Ok(0)
        } else {
            Err(anyhow!(errors)).with_context(|| "Not everything went as expected.")
        }
    }
}
