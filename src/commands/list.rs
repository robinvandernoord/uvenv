use std::path::PathBuf;

use crate::cli::{ListOptions, Process};
use crate::helpers::ResultToString;
use crate::metadata::{get_venv_dir, Metadata};

impl Process for ListOptions {
    fn process(self) -> Result<u32, String> {
        let venv_dir_path = get_venv_dir();
        let possibly_missing = std::fs::read_dir(&venv_dir_path);

        let must_exist = match possibly_missing {
            Ok(dir) => dir,
            Err(_) => {
                std::fs::create_dir_all(&venv_dir_path).map_err_to_string()?;
                std::fs::read_dir(&venv_dir_path).map_err_to_string()?
            }
        };

        for _dir in must_exist {
            if let Ok(dir) = _dir {
                if let Some(metadata) = Metadata::for_dir(&dir.path()) {

                    // todo: format human short
                    // todo: format verbose (struct dump)
                    // todo: format json
                    //        + short/verbose ?

                    println!("{}", &metadata.format_human());
                }
            }
        }

        return Ok(0);
    }
}
