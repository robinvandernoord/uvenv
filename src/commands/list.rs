use std::fs::ReadDir;

use crate::cli::{ListOptions, Process};
use crate::helpers::ResultToString;
use crate::metadata::{get_venv_dir, Metadata};
use owo_colors::OwoColorize;

async fn read_from_folder(metadata_dir: ReadDir) -> Vec<Metadata> {
    let mut result: Vec<Metadata> = vec![];

    for dir in metadata_dir.flatten() {
        if let Some(metadata) = Metadata::for_dir(&dir.path(), true).await {
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
        items: &Vec<&Metadata>,
    ) -> Result<i32, String> {
        let json = if self.short {
            serde_json::to_string(items).map_err_to_string()?
        } else {
            serde_json::to_string_pretty(items).map_err_to_string()?
        };

        println!("{}", json);

        Ok(0)
    }
}

pub async fn list_packages() -> Result<Vec<Metadata>, String> {
    let venv_dir_path = get_venv_dir();
    let possibly_missing = std::fs::read_dir(&venv_dir_path);

    let must_exist = match possibly_missing {
        Ok(dir) => dir,
        Err(_) => {
            std::fs::create_dir_all(&venv_dir_path).map_err_to_string()?;
            std::fs::read_dir(&venv_dir_path).map_err_to_string()?
        },
    };

    let result = read_from_folder(must_exist).await;
    Ok(result)
}

impl Process for ListOptions {
    async fn process(self) -> Result<i32, String> {
        let all_items = list_packages().await?;

        let filtered_items: Vec<&Metadata> = if !self.venv_names.is_empty() {
            // add filter
            all_items
                .iter()
                .filter(|k| self.venv_names.contains(&k.name))
                .collect()
        } else {
            all_items.iter().collect() // iter collect to make it the same type as the other branch...
        };

        if self.json {
            return self.process_json(&filtered_items).await;
        }

        for metadata in filtered_items {
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
