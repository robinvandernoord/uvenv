use std::fs::ReadDir;

use owo_colors::OwoColorize;

use crate::cli::{ListOptions, Process};
use crate::commands::self_version::{is_latest, uvenv_version};
use crate::metadata::{get_venv_dir, LoadMetadataConfig, Metadata};
use crate::promises::handle_promises;
use crate::pypi::get_latest_version;
use crate::uv::uv_search_python;

async fn read_from_folder_filtered(
    metadata_dir: ReadDir,
    config: &LoadMetadataConfig,
    filter_names: &[String],
) -> Vec<Metadata> {
    let (cap, _) = metadata_dir.size_hint(); // estimate size
    let mut promises = Vec::with_capacity(cap);

    for dir in metadata_dir.flatten() {
        let venv_name = dir.file_name().into_string().unwrap_or_default();

        if !filter_names.is_empty() && !filter_names.contains(&venv_name) {
            continue;
        }

        promises.push(
            Metadata::for_owned_dir(dir.path(), config), // no .await so its a future (requires ownership of dir_path)
        );
    }

    handle_promises(promises).await
}

impl ListOptions {
    pub fn process_json(
        self,
        items: &[Metadata],
    ) -> anyhow::Result<i32> {
        let json = if self.short {
            serde_json::to_string(items)?
        } else {
            serde_json::to_string_pretty(items)?
        };

        println!("{json}");

        Ok(0)
    }

    pub const fn to_metadataconfig(&self) -> LoadMetadataConfig {
        LoadMetadataConfig {
            recheck_scripts: true, // always done
            updates_check: !self.skip_updates,
            updates_prereleases: self.show_prereleases,
            updates_ignore_constraints: self.ignore_constraints,
        }
    }
}

async fn is_uvenv_outdated(silent: bool) -> bool {
    let latest = get_latest_version("uvenv", true, None).await;

    // uvenv version comes from Cargo.toml
    let version = uvenv_version();

    let is_outdated = !is_latest(version, latest.as_ref());

    if is_outdated && !silent {
        if let Some(latest_version) = latest {
            eprintln!(
                "{} ({} < {})",
                "uvenv is outdated!".yellow(),
                version.red(),
                latest_version.to_string().green()
            );
        }
    }

    is_outdated
}

pub async fn list_packages(
    config: &LoadMetadataConfig,
    filter_names: Option<&[String]>,
    python: Option<&String>,
) -> anyhow::Result<Vec<Metadata>> {
    let venv_dir_path = get_venv_dir();

    // no tokio::fs because ReadDir.flatten doesn't exist then.
    let must_exist = if let Ok(dir) = std::fs::read_dir(&venv_dir_path) {
        dir
    } else {
        std::fs::create_dir_all(&venv_dir_path)?;
        std::fs::read_dir(&venv_dir_path)?
    };

    let mut results =
        read_from_folder_filtered(must_exist, config, filter_names.unwrap_or(&[])).await;

    if let Some(python_filter) = uv_search_python(python).await {
        results.retain(|meta| meta.python_raw == python_filter);
    }
    Ok(results)
}

impl Process for ListOptions {
    async fn process(self) -> anyhow::Result<i32> {
        if self.venv_names.is_empty() {
            // don't show uvenv version warning if package names were supplied
            is_uvenv_outdated(false).await;
        }

        let config = self.to_metadataconfig();

        let items = list_packages(&config, Some(&self.venv_names), self.python.as_ref()).await?;

        if self.json {
            return self.process_json(&items);
        }

        for metadata in items {
            if self.verbose {
                // println!("{}", dbg_pls::color(&metadata));
                print!("{}", &metadata.format_debug());
            } else if self.short {
                print!("{}", &metadata.format_short());
            } else {
                print!("{}", &metadata.format_human()?);
            }
        }

        Ok(0)
    }
}
