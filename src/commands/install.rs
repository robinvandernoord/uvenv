use std::path::{Path, PathBuf};
use std::str::FromStr;
use crate::cli::{InstallOptions, Process};
use crate::metadata::get_venv_dir;
use pep508_rs::Requirement;

pub fn create_new_venv(package_name: &str) -> PathBuf {
    let venvs = get_venv_dir();
    return venvs.join(package_name);
}

pub fn ensure_venv(maybe_venv: Option<&Path>, requirement: &Requirement) -> PathBuf {
    let venv_path = match maybe_venv {
        Some(venv) => venv.to_path_buf(), // todo: ensure exists
        None => create_new_venv(&requirement.name.to_string()) // todo: actually create
    };

    // uv venv {venv_path}

    return venv_path;
}


pub fn install_package(
    package_name: &str,
    maybe_venv: Option<&Path>,
    python: Option<String>,
    force: bool,
    extras: Vec<&str>,
    no_cache: bool,
) -> Result<String, String> {
    let requirement = Requirement::from_str(package_name).unwrap();
    // requirement.name
    // requirement.extras
    // requirement.version_or_url
    
    let venv_path = ensure_venv(maybe_venv, &requirement);


    dbg!(requirement);
    dbg!(venv_path);

    Ok(String::from("todo"))
}


impl Process for InstallOptions {
    fn process(self) -> Result<u32, String> {
        match install_package(
            &self.package_name, 
            None, 
            self.python, 
            self.force, 
            vec![], 
            self.no_cache,
        ) {
            Ok(msg) => {
                println!("{}", msg);
                return Ok(0)
            },
            Err(msg) => {
                return Err(msg)
            }
        }
    }
}
