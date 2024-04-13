use std::fs::ReadDir;

use crate::cli::{ListOptions, Process};
use crate::helpers::ResultToString;
use crate::metadata::{get_venv_dir, Metadata};
use owo_colors::OwoColorize;

impl ListOptions {
    pub fn process_json(self, must_exist: ReadDir) -> Result<u32, String> {
        let mut result: Vec<Metadata> = vec![];

        for _dir in must_exist {
            if let Ok(dir) = _dir {
                if let Some(metadata) = Metadata::for_dir(&dir.path()) {
                    result.push(metadata)
                }
            }
        }

        let json: String;

        if self.short {
            json = serde_json::to_string(&result).map_err_to_string()?;
        } else {
            json = serde_json::to_string_pretty(&result).map_err_to_string()?;
        }

        print!("{}", json);

        Ok(0)
    }
}

impl Process for ListOptions {
    async fn process(self) -> Result<u32, String> {
        let venv_dir_path = get_venv_dir();
        let possibly_missing = std::fs::read_dir(&venv_dir_path);

        let must_exist = match possibly_missing {
            Ok(dir) => dir,
            Err(_) => {
                std::fs::create_dir_all(&venv_dir_path).map_err_to_string()?;
                std::fs::read_dir(&venv_dir_path).map_err_to_string()?
            }
        };

        if self.json {
            return self.process_json(must_exist);
        }

        for _dir in must_exist {
            if let Ok(dir) = _dir {
                if let Some(metadata) = Metadata::for_dir(&dir.path()) {
                    if self.verbose {
                        println!("{}", dbg_pls::color(&metadata));
                    } else if self.short {
                        println!("{}", &metadata.format_short());
                    } else {
                        println!("{}", &metadata.format_human());
                    }
                } else {
                    // todo: better logging
                    let venv_name = dir.file_name().into_string().unwrap_or_default();

                    eprintln!("! metadata for '{}' could not be read!", venv_name.red());
                }
            }
        }

        return Ok(0);
    }
}
