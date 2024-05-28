use std::fs::ReadDir;

use crate::cli::{ListOptions, Process};
use crate::helpers::ResultToString;
use crate::metadata::{get_venv_dir, LoadMetadataConfig, Metadata};
use owo_colors::OwoColorize;

async fn read_from_folder(
    metadata_dir: ReadDir,
    config: &LoadMetadataConfig,
) -> Vec<Metadata> {
    let mut result: Vec<Metadata> = vec![];

    for dir in metadata_dir.flatten() {
        if let Some(metadata) = Metadata::for_dir(&dir.path(), config).await {
            result.push(metadata)
        } else {
            let venv_name = dir.file_name().into_string().unwrap_or_default();

            eprintln!("! metadata for '{}' could not be read!", venv_name.red());
        }
    }

    result
}

impl ListOptions {
    pub async fn process_json(
        self,
        items: &Vec<Metadata>,
    ) -> Result<i32, String> {
        let json = if self.short {
            serde_json::to_string(items).map_err_to_string()?
        } else {
            serde_json::to_string_pretty(items).map_err_to_string()?
        };

        println!("{}", json);

        Ok(0)
    }

    pub fn to_metadataconfig(&self) -> LoadMetadataConfig {
        LoadMetadataConfig {
            recheck_scripts: true, // always done
            updates_check: !self.skip_updates,
            updates_prereleases: self.show_prereleases,
            updates_ignore_constraints: self.ignore_constraints,
        }
    }
}

pub async fn list_packages(config: &LoadMetadataConfig) -> Result<Vec<Metadata>, String> {
    let venv_dir_path = get_venv_dir();
    let possibly_missing = std::fs::read_dir(&venv_dir_path);

    // no tokio::fs because ReadDir.flatten doesn't exist then.
    let must_exist = match possibly_missing {
        Ok(dir) => dir,
        Err(_) => {
            std::fs::create_dir_all(&venv_dir_path).map_err_to_string()?;
            std::fs::read_dir(&venv_dir_path).map_err_to_string()?
        },
    };

    Ok(read_from_folder(must_exist, config).await)
}

impl Process for ListOptions {
    async fn process(self) -> Result<i32, String> {
        let config = self.to_metadataconfig();

        let mut items = list_packages(&config).await?;

        if !self.venv_names.is_empty() {
            items.retain(|k| self.venv_names.contains(&k.name))
        }

        if self.json {
            return self.process_json(&items).await;
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
