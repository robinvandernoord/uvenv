use crate::cmd::{find_sibling, run, run_print_output};
use anyhow::{Context, anyhow, bail};
use core::fmt::Write;
use directories::ProjectDirs;
use itertools::Itertools;
use owo_colors::OwoColorize;
use pushd::Pushd;
use std::ffi::OsStr;
use std::path::Path;
use std::{collections::HashSet, path::PathBuf};
use uv_cache::Cache;
use uv_client::{BaseClientBuilder, Connectivity};
use uv_distribution_types::{InstalledDistKind, Name};
use uv_installer::SitePackages;
use uv_normalize::PackageName;
use uv_pep508::Requirement;
use uv_preview::Preview;
use uv_python::{
    EnvironmentPreference, Interpreter, PythonDownloads, PythonEnvironment, PythonInstallation,
    PythonPreference, PythonRequest,
};

use uv_pep508::VersionOrUrl::VersionSpecifier;

use crate::helpers::{PathToString, set_env_var};
use crate::metadata::get_work_dir;

pub async fn maybe_get_uv_binary() -> Option<String> {
    find_sibling("uv").await.map(PathToString::to_string)
}

pub async fn get_uv_binary() -> String {
    maybe_get_uv_binary().await.unwrap_or_else(
        // fallback, hope 'uv' is available in global scope:
        || String::from("uv"),
    )
}

fn apply_uv_env_settings() {
    if cfg!(feature = "snap") {
        let work_dir = get_work_dir();
        let python_dir = work_dir.join("python");

        // by default, uv in snap would install at
        // ~/snap/uvenv/<revision>/.local/share/uv/python/
        // meaning it would be moved after each update;
        // leading to longer update times and breaking symlinks.
        // so, we set the install dir to a fixed location (~/snap/uvenv/common/python)
        set_env_var("UV_PYTHON_INSTALL_DIR", python_dir.to_string());
    }
}

/// Start `uv` in a subprocess and handle its output
/// Note: while `uv::main` exists, it's not recommended to use as an entrypoint.
/// It also calls `exit`, stopping `uvenv` instead of returning an exit code.
pub async fn uv<S: AsRef<OsStr>>(args: &[S]) -> anyhow::Result<bool> {
    // venv could be unavailable, use 'uv' from this library's requirement
    let script = get_uv_binary().await;
    apply_uv_env_settings();

    let subcommand = args
        .first()
        .ok_or_else(|| anyhow!("No subcommand passed"))?
        .as_ref()
        .to_str()
        .unwrap_or_default(); // cursed but makes it work with both &str and String
    let err_prefix = format!("uv {subcommand}");

    // temporarily (until end of scope) change cwd
    // to `~/.config/uvenv` to prevent reading local pyproject.toml.
    // this replaces `set_env_var("UV_NO_CONFIG", "1")`,
    // which would also ignore ~/.config/uv/uv.toml
    let _pd = Pushd::new(get_work_dir());

    run(&script, args, Some(err_prefix)).await
    // _pd is dropped, original cwd is restored
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

fn uv_featureflags() -> Preview {
    Preview::default()
}

/// try to find a `PythonEnvironment` based on Cache or currently active virtualenv (`VIRTUAL_ENV`).
pub fn uv_venv(maybe_cache: Option<Cache>) -> anyhow::Result<PythonEnvironment> {
    let cache = maybe_cache.unwrap_or_else(uv_cache);
    cache.venv_dir()?; // set up the cache

    let environ = PythonEnvironment::find(
        &PythonRequest::Any,                // just find me a python
        EnvironmentPreference::OnlyVirtual, // venv is always virtual
        PythonPreference::Managed,
        &cache,
        uv_featureflags(),
    )?;

    Ok(environ)
}

/// try to find a `PythonEnvironment` based on a specific Python path (as str)
pub fn environment_from_path_str(path: &str) -> anyhow::Result<PythonEnvironment> {
    let cache = uv_cache();

    Ok(PythonEnvironment::find(
        &PythonRequest::parse(path),
        EnvironmentPreference::ExplicitSystem, // based on above python wishes
        PythonPreference::Managed,
        &cache,
        uv_featureflags(),
    )?)
}

/// try to find a `PythonEnvironment` based on a specific Python path (as Path)
pub fn environment_from_path(path: &Path) -> anyhow::Result<PythonEnvironment> {
    environment_from_path_str(path.to_str().unwrap_or_default())
}

/// try to find a `PythonEnvironment` based on the System python
pub fn system_environment() -> anyhow::Result<PythonEnvironment> {
    let cache = uv_cache();

    Ok(PythonEnvironment::find(
        &PythonRequest::Any, // just find me a python
        EnvironmentPreference::OnlySystem,
        PythonPreference::OnlySystem,
        &cache,
        uv_featureflags(),
    )?)
}

fn uv_offline_client() -> BaseClientBuilder<'static> {
    BaseClientBuilder::default()
        .connectivity(Connectivity::Offline)
        .native_tls(false)
}

/// e.g. 3.12 -> /usr/lib/python3.12, to match with `metadata.python_raw`
pub async fn uv_search_python(python: Option<&str>) -> Option<String> {
    let interpreter_request = python.map(PythonRequest::parse);

    let python_request = interpreter_request.as_ref()?; // exit early

    let cache = uv_cache();
    let client = uv_offline_client();

    // Locate the Python interpreter to use in the environment
    let python_installation = PythonInstallation::find_or_download(
        Some(python_request),
        EnvironmentPreference::OnlySystem,
        PythonPreference::OnlySystem,
        PythonDownloads::Never,
        &client,
        &cache,
        None,
        None,
        None,
        None,
        uv_featureflags(),
    )
    .await
    .ok()?;

    let interpreter = python_installation.into_interpreter();

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
    }

    bail!(
        "No version found for '{}'.",
        package_name.to_string().yellow()
    )
}

pub fn uv_freeze(python: &PythonEnvironment) -> anyhow::Result<String> {
    // variant with BTree return type is also possible, but everything is currently built on
    // the `pip freeze` output string format, so use that for now:
    let mut result = String::new();

    // logic below was copied from the `uv pip freeze` command source code:
    // https://github.com/astral-sh/uv/blob/c9787f9fd80c58f1242bee5123919eb16f4b05c1/crates/uv/src/commands/pip/freeze.rs

    // Build the installed index.
    let site_packages = SitePackages::from_environment(python)?;
    for installed_dist in site_packages
        .iter()
        // .filter() ?
        .sorted_unstable_by(|one, two| one.name().cmp(two.name()).then(one.version().cmp(two.version())))
    {
        match &installed_dist.kind {
            InstalledDistKind::Registry(dist) => {
                // result.push_str(&format!("{}=={}\n", dist.name(), dist.version));
                writeln!(result, "{}=={}", dist.name(), dist.version)?;
            },
            InstalledDistKind::Url(dist) => {
                if dist.editable {
                    // result.push_str(&format!("-e {}\n", dist.url));
                    writeln!(result, "-e {}", dist.url)?;
                } else {
                    // result.push_str(&format!("{} @ {}\n", dist.name(), dist.url));
                    writeln!(result, "{} @ {}", dist.name(), dist.url)?;
                }
            },
            InstalledDistKind::EggInfoFile(dist) => {
                // result.push_str(&format!("{}=={}\n", dist.name(), dist.version));
                writeln!(result, "{}=={}", dist.name(), dist.version)?;
            },
            InstalledDistKind::EggInfoDirectory(dist) => {
                // result.push_str(&format!("{}=={}\n", dist.name(), dist.version));
                writeln!(result, "{}=={}", dist.name(), dist.version)?;
            },
            InstalledDistKind::LegacyEditable(dist) => {
                // result.push_str(&format!("-e {}\n", dist.target.display()));
                writeln!(result, "-e {}", dist.target.display())?;
            },
        }
    }

    Ok(result)
}

#[expect(
    dead_code,
    reason = "Required for `uv_freeze` (but that function is currently unused)"
)]
#[derive(Debug, Clone)]
pub enum PythonSpecifier<'src> {
    Path(&'src Path),
    PathBuf(&'src PathBuf),
    Str(&'src str),
    String(String),
    Environ(PythonEnvironment),
}

impl PythonSpecifier<'_> {
    pub fn into_environment(self) -> anyhow::Result<PythonEnvironment> {
        match self {
            PythonSpecifier::Path(path) => environment_from_path(path),
            PythonSpecifier::PathBuf(buf) => environment_from_path(buf.as_path()),
            PythonSpecifier::Str(string) => environment_from_path_str(string),
            PythonSpecifier::String(string) => environment_from_path_str(&string),
            PythonSpecifier::Environ(env) => Ok(env),
        }
    }
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
        stdlib.unwrap_or_default().to_owned()
    }
}

pub trait ExtractInfo {
    fn version(&self) -> String;
    fn extras(&self) -> HashSet<String>;
}

impl ExtractInfo for Requirement {
    fn version(&self) -> String {
        match &self.version_or_url {
            Some(VersionSpecifier(version_specifier)) => version_specifier.to_string(),
            _ => String::new(),
        }
    }

    fn extras(&self) -> HashSet<String> {
        self.extras.iter().map(ToString::to_string).collect()
    }
}
