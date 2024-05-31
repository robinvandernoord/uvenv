use std::fs::ReadDir;

use crate::cli::{ListOptions, Process};
use crate::helpers::ResultToString;
use crate::metadata::{get_venv_dir, LoadMetadataConfig, Metadata};
use owo_colors::OwoColorize;

async fn read_from_folder_filtered(
    metadata_dir: ReadDir,
    config: &LoadMetadataConfig,
    filter_names: &Vec<String>,
) -> Vec<Metadata> {
    let mut result: Vec<Metadata> = vec![];

    for dir in metadata_dir.flatten() {
        let venv_name = dir.file_name().into_string().unwrap_or_default();

        if !filter_names.is_empty() && !filter_names.contains(&venv_name) {
            continue;
        }

        if let Some(metadata) = Metadata::for_dir(&dir.path(), config).await {
            result.push(metadata);
        } else {
            eprintln!("! metadata for '{}' could not be read!", venv_name.red());
        }
    }

    result
}

impl ListOptions {
    pub fn process_json(
        self,
        items: &Vec<Metadata>,
    ) -> Result<i32, String> {
        let json = if self.short {
            serde_json::to_string(items).map_err_to_string()?
        } else {
            serde_json::to_string_pretty(items).map_err_to_string()?
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
    filter_names: Option<&Vec<String>>,
) -> Result<Vec<Metadata>, String> {
    let venv_dir_path = get_venv_dir();

    // no tokio::fs because ReadDir.flatten doesn't exist then.
    let must_exist = if let Ok(dir) = std::fs::read_dir(&venv_dir_path) {
        dir
    } else {
        std::fs::create_dir_all(&venv_dir_path).map_err_to_string()?;
        std::fs::read_dir(&venv_dir_path).map_err_to_string()?
    };

    Ok(read_from_folder_filtered(must_exist, config, filter_names.unwrap_or(&vec![])).await)
}

impl Process for ListOptions {
    async fn process(self) -> Result<i32, String> {
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
