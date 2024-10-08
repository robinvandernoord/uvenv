use anyhow::{bail, Context};
use owo_colors::OwoColorize;
use std::path::{Path, PathBuf};
use uv_pep508::Requirement;

use uv_python::PythonEnvironment;

use crate::cli::{Process, RunOptions};
use crate::commands::install::_install_package;
use crate::commands::runpython::process_subprocess;
use crate::helpers::PathAsStr;
use crate::pip::parse_requirement;
use crate::symlinks::find_symlinks;
use crate::uv::uv_get_installed_version;
use crate::venv::{activate_venv, create_venv, remove_venv};
use core::fmt::Write;

async fn _find_executable(
    requirement: &Requirement,
    package_spec: &str,
    venv: &PythonEnvironment,
) -> anyhow::Result<String> {
    let installed_version = uv_get_installed_version(&requirement.name, Some(venv))?;
    let mut symlinks = find_symlinks(requirement, &installed_version, venv).await;

    match symlinks.len() {
        0 => {
            // just return the original name just as a last hope:
            Ok(requirement.name.to_string())
        },
        1 => Ok(symlinks
            .pop()
            .expect("Popping should always work if len == 1!")),
        _ => {
            // too many choices, user should provide --binary <something>
            let mut related = String::new();

            for option in symlinks {
                if package_spec == option {
                    // exact match -> probably what you want!
                    return Ok(option);
                }

                let code = format!("uvenv run {package_spec} --binary {option} ...");
                // related.push_str(&format!("\t- {} | `{}` \n", option.green(), code.blue()));
                writeln!(related, "\t- {} | `{}` ", option.green(), code.blue())?;
            }

            bail!("'{}' executable not found for install spec '{}'.\nMultiple related scripts were found:\n{}",
                        requirement.name.to_string().green(),
                        package_spec.green(),
                        related,
                )
        },
    }
}

pub async fn find_executable(
    requirement: &Requirement,
    binary: Option<&String>,
    package_spec: &str,
    venv: &PythonEnvironment,
    venv_path: &Path,
) -> anyhow::Result<PathBuf> {
    let executable = match binary {
        Some(executable) => executable.to_owned(),
        None => _find_executable(requirement, package_spec, venv).await?,
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
    args: &[String],
) -> anyhow::Result<i32> {
    let full_exec_path =
        find_executable(requirement, binary, package_spec, venv, venv_path).await?;

    process_subprocess(full_exec_path.as_path(), args)
}
pub async fn run_package<S: AsRef<str>>(
    package_spec: &str,
    python: Option<&String>,
    keep: bool,
    no_cache: bool,
    binary: Option<&String>,
    inject: &[S],
    args: &[String],
) -> anyhow::Result<i32> {
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
        Some(String::from("/tmp/uvenv-")),
    )
    .await?;

    let venv_name = &venv_path.as_str();

    if keep {
        eprintln!("ℹ️ Using virtualenv {}", venv_name.blue());
    }

    // ### 2 ###
    let venv = &activate_venv(venv_path).await?;

    // already expects activated venv:
    _install_package(package_spec, inject, no_cache, false, false).await?;

    // ### 3 ###
    let result = run_executable(&requirement, binary, package_spec, venv, venv_path, args).await;

    // ### 4 ###

    if !keep {
        // defer! not possible here because of await
        remove_venv(venv_path).await?;
    }

    result
}

impl Process for RunOptions {
    async fn process(self) -> anyhow::Result<i32> {
        run_package(
            &self.package_name,
            self.python.as_ref(),
            self.keep,
            self.no_cache,
            self.binary.as_ref(),
            &self.with,
            &self.args,
        )
        .await
        .with_context(|| {
            format!(
                "Something went wrong while trying to run '{}';",
                &self.package_name
            )
        })
    }
}
