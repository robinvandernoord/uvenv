use anyhow::{bail, Context};
use std::ffi::OsStr;
use std::{collections::HashSet, path::PathBuf};

use crate::cmd::{find_sibling, run, run_print_output};
use directories::ProjectDirs;
use owo_colors::OwoColorize;
use pep508_rs::{PackageName, Requirement};
use uv_cache::Cache;
use uv_client::{BaseClientBuilder, Connectivity};
use uv_installer::SitePackages;
use uv_python::{
    EnvironmentPreference, Interpreter, PythonDownloads, PythonEnvironment, PythonInstallation,
    PythonPreference, PythonRequest,
};

use crate::helpers::PathToString;

pub async fn _get_uv_binary() -> Option<String> {
    // if bundled with entrypoint:
    // arg 0 = python
    // arg 1 = .../bin/uvenv
    // elif bundled as bin, use current_exe (because arg 0 is just 'uvenv' instead of a path):

    // let Some(binary) = env::args().nth(0) else {
    //     return None;
    // };
    // let Ok(binary_path) = PathBuf::from_str(&binary) else {
    //     return None;
    // };

    find_sibling("uv").await.map(PathToString::to_string)
}

pub async fn get_uv_binary() -> String {
    _get_uv_binary().await.unwrap_or_else(
        // fallback, hope 'uv' is available in global scope:
        || String::from("uv"),
    )
}

pub async fn uv<S>(args: &[S]) -> anyhow::Result<bool>
where
    S: AsRef<OsStr>,
{
    // venv could be unavailable, use 'uv' from this library's requirement
    let script = get_uv_binary().await;

    let subcommand = &args[0].as_ref().to_str().unwrap_or_default(); // cursed but makes it work with both &str and String
    let err_prefix = format!("uv {subcommand}");

    run(&script, args, Some(err_prefix)).await
}

pub async fn uv_with_output<S: AsRef<OsStr>>(args: &[S]) -> anyhow::Result<i32> {
    let script = get_uv_binary().await;
    run_print_output(script, args).await
}

pub fn uv_cache() -> Cache {
    ProjectDirs::from("", "", "uv").map_or_else(
        || Cache::from_path(".uv_cache"),
        |project_dirs| Cache::from_path(project_dirs.cache_dir()),
    )
}

/// try to find an `PythonEnvironment` based on Cache or currently active virtualenv (`VIRTUAL_ENV`).
pub fn uv_venv(maybe_cache: Option<Cache>) -> anyhow::Result<PythonEnvironment> {
    let cache = maybe_cache.unwrap_or_else(uv_cache);
    cache.environment()?; // set up the cache

    let environ = PythonEnvironment::find(
        &PythonRequest::Any,                // just find me a python
        EnvironmentPreference::OnlyVirtual, // venv is always virtual
        &cache,
    )?;

    Ok(environ)
}

fn uv_offline_client() -> BaseClientBuilder<'static> {
    BaseClientBuilder::default()
        .connectivity(Connectivity::Offline)
        .native_tls(false)
}

/// e.g. 3.12 -> /usr/lib/python3.12, to match with `metadata.python_raw`
pub async fn uv_search_python(python: Option<&String>) -> Option<String> {
    let interpreter_request =
        python.map(|requested_version| PythonRequest::parse(requested_version));

    let interpreter_request = interpreter_request.as_ref()?; // exit early

    let cache = uv_cache();
    let client = uv_offline_client();

    // Locate the Python interpreter to use in the environment
    let python = PythonInstallation::find_or_download(
        Some(interpreter_request),
        EnvironmentPreference::OnlySystem,
        PythonPreference::OnlySystem,
        PythonDownloads::Never,
        &client,
        &cache,
        None,
    )
    .await
    .ok()?;

    let interpreter = python.into_interpreter();

    Some(interpreter.stdlib_as_string())
}

pub fn uv_get_installed_version(
    package_name: &PackageName,
    maybe_venv: Option<&PythonEnvironment>,
) -> anyhow::Result<String> {
    let environment: PythonEnvironment; // lifetime for if maybe_venv is None

    let site_packages = if let Some(venv) = maybe_venv {
        SitePackages::from_environment(venv)
    } else {
        environment =
            uv_venv(None).with_context(|| format!("{}", "Failed to set up venv!".red()))?;
        SitePackages::from_environment(&environment)
    }
    .ok();

    if let Some(pkgs) = site_packages {
        // for result in pkgs.get_packages(package_name) {
        if let Some(result) = pkgs.get_packages(package_name).into_iter().next() {
            return Ok(result.version().to_string());
        }
    };

    bail!(
        "No version found for '{}'.",
        package_name.to_string().yellow()
    )
}

pub trait Helpers {
    fn to_path_buf(&self) -> PathBuf;
    fn stdlib_as_string(&self) -> String;
}

impl Helpers for PythonEnvironment {
    fn to_path_buf(&self) -> PathBuf {
        self.root().to_path_buf()
    }

    fn stdlib_as_string(&self) -> String {
        self.interpreter().stdlib_as_string()
    }
}

impl Helpers for Interpreter {
    fn to_path_buf(&self) -> PathBuf {
        self.stdlib().to_path_buf()
    }

    fn stdlib_as_string(&self) -> String {
        let stdlib = self.stdlib().to_str();
        stdlib.unwrap_or_default().to_string()
    }
}

pub trait ExtractInfo {
    fn version(&self) -> String;
    fn extras(&self) -> HashSet<String>;
}

impl ExtractInfo for Requirement {
    fn version(&self) -> String {
        match self.version_or_url.clone() {
            Some(pep508_rs::VersionOrUrl::VersionSpecifier(v)) => v.to_string(),
            _ => String::new(),
        }
    }

    fn extras(&self) -> HashSet<String> {
        self.extras.iter().map(ToString::to_string).collect()
    }
}
