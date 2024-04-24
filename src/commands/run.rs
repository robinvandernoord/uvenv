use owo_colors::OwoColorize;
use pep508_rs::Requirement;
use std::path::{Path, PathBuf};

use uv_interpreter::PythonEnvironment;

use crate::cli::{Process, RunOptions};
use crate::commands::install::_install_package;
use crate::commands::runpython::process_subprocess;
use crate::pip::parse_requirement;
use crate::symlinks::find_symlinks;
use crate::uv::uv_get_installed_version;
use crate::venv::{activate_venv, create_venv, remove_venv};

pub async fn find_executable(
    requirement: &Requirement,
    binary: Option<&String>,
    package_spec: &str,
    venv: &PythonEnvironment,
    venv_path: &Path,
) -> Result<PathBuf, String> {
    let executable = match binary {
        Some(executable) => executable.to_owned(),
        None => {
            let installed_version = uv_get_installed_version(&requirement.name, Some(venv))?;
            let symlinks = find_symlinks(requirement, &installed_version, venv).await;

            match symlinks.len() {
                0 => {
                    // just return the original name just as a last hope:
                    requirement.name.to_string()
                },
                1 => symlinks[0].to_owned(),
                _ => {
                    // too many choices, user should provide --binary <something>
                    let mut related = String::new();

                    for option in symlinks {
                        let code = format!("uvx run {} --binary {} ...", package_spec, option);
                        related.push_str(&format!("\t- {} | `{}` \n", option.green(), code.blue()));
                    }

                    return Err(
                        format!("'{}' executable not found for install spec '{}'.\nMultiple related scripts were found:\n{}",
                                requirement.name.to_string().green(),
                                package_spec.green(),
                                related,
                        )
                    );
                },
            }
        },
    };

    let full_exec_path = venv_path.join("bin").join(executable);
    Ok(full_exec_path)
}

pub async fn run_executable(
    requirement: &Requirement,
    binary: Option<&String>,
    package_spec: &str,
    venv: &PythonEnvironment,
    venv_path: &Path,
    args: Vec<String>,
) -> Result<i32, String> {
    let full_exec_path =
        find_executable(requirement, binary, package_spec, venv, venv_path).await?;

    process_subprocess(full_exec_path.as_path(), &args)
}
pub async fn run_package(
    package_spec: &str,
    python: Option<&String>,
    keep: bool,
    no_cache: bool,
    binary: Option<&String>,
    args: Vec<String>,
) -> Result<i32, String> {
    // 1. create a temp venv
    // 2. install package
    // 3. run 'binary' or find runnable(s) in package
    // 4. if not 'keep': remove temp venv

    // ### 1 ###

    let (requirement, _) = parse_requirement(package_spec).await?;

    let venv_path = &create_venv(
        &requirement.name,
        python,
        true,
        true,
        Some(String::from("/tmp/uvx-")),
    )
    .await?;

    let venv_name = &venv_path.to_str().unwrap_or_default();

    if keep {
        eprintln!("ℹ️ Using virtualenv {}", venv_name.blue())
    }

    // ### 2 ###
    let venv = &activate_venv(venv_path).await?;

    // already expects activated venv:
    _install_package(package_spec, &Vec::new(), no_cache, false, false).await?;

    // ### 3 ###
    let result = run_executable(&requirement, binary, package_spec, venv, venv_path, args).await;

    // ### 4 ###

    if !keep {
        remove_venv(venv_path).await?;
    }

    result
}

impl Process for RunOptions {
    async fn process(self) -> Result<i32, String> {
        match run_package(
            &self.package_name,
            self.python.as_ref(),
            self.keep,
            self.no_cache,
            self.binary.as_ref(),
            self.args,
        )
        .await
        {
            Ok(code) => Ok(code),
            Err(msg) => Err(msg),
        }
    }
}
