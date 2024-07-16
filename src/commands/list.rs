use std::fs::ReadDir;

use crate::cli::{ListOptions, Process};
use crate::metadata::{get_venv_dir, LoadMetadataConfig, Metadata};
use crate::promises::handle_promises;

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

pub async fn list_packages(
    config: &LoadMetadataConfig,
    filter_names: Option<&[String]>,
) -> anyhow::Result<Vec<Metadata>> {
    let venv_dir_path = get_venv_dir();

    // no tokio::fs because ReadDir.flatten doesn't exist then.
    let must_exist = if let Ok(dir) = std::fs::read_dir(&venv_dir_path) {
        dir
    } else {
        std::fs::create_dir_all(&venv_dir_path)?;
        std::fs::read_dir(&venv_dir_path)?
    };

    Ok(read_from_folder_filtered(must_exist, config, filter_names.unwrap_or(&[])).await)
}

impl Process for ListOptions {
    async fn process(self) -> anyhow::Result<i32> {
        let config = self.to_metadataconfig();

        let items = list_packages(&config, Some(&self.venv_names)).await?;

        if self.json {
            return self.process_json(&items);
        }

        for metadata in items {
            if self.verbose {
                // println!("{}", dbg_pls::color(&metadata));
                println!("{:#?}", &metadata);
            } else if self.short {
                println!("{}", &metadata.format_short());
            } else {
                println!("{}", &metadata.format_human());
            }
        }

        Ok(0)
    }
}
