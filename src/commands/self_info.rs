use anyhow::bail;
use core::fmt::Write as _;
use core::str::FromStr;
use futures::future;
use owo_colors::OwoColorize;
use std::collections::BTreeMap;
use std::env;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use uv_pep440::Version;

use crate::cli::{Process, SelfInfoOptions};
use crate::cmd::run_get_output;
use crate::commands::self_update::{find_python, get_package_versions_pip};
use crate::helpers::{PathAsStr, PathToString};
use crate::metadata::{get_bin_dir, get_work_dir};
use crate::pypi::get_latest_version;
use crate::uv::get_uv_binary;

pub const CURRENT_UVENV_VERSION: &str = env!("CARGO_PKG_VERSION");

#[expect(dead_code, reason = "Could still be useful in the future.")]
async fn get_latest_versions(package_names: Vec<&str>) -> BTreeMap<&str, Option<Version>> {
    let promises: Vec<_> = package_names
        .iter()
        .map(|it| get_latest_version(it, true, None))
        .collect();
    let resolved = future::join_all(promises).await;

    let mut result = BTreeMap::new();
    for (package, version) in package_names.into_iter().zip(resolved.into_iter()) {
        result.insert(package, version);
    }

    result
}

fn red_or_green(
    text: &str,
    ok: bool,
) -> String {
    if ok {
        format!("{}", text.green())
    } else {
        format!("{}", text.red())
    }
}

// separate, public function for testing
pub fn compare_versions(
    current: &str,
    latest: &str,
) -> bool {
    if latest == "?" || current == "?" {
        return false;
    }

    // should compare uv_pep440::version::Version instead of str:

    let Ok(current_version) = Version::from_str(current) else {
        // if this fails, it's probably not up to date ->
        return false;
    };

    let Ok(latest_version) = Version::from_str(latest) else {
        // if this fails, we can't know if it's up to date ->
        return true;
    };

    current_version.ge(&latest_version)
}

pub fn is_latest(
    current: &str,
    latest: Option<&Version>,
) -> bool {
    let Some(version) = latest else { return false };

    compare_versions(current, &version.to_string())
}

/// Check if path exists and is executable (Result variant)
fn try_is_executable(path: &Path) -> anyhow::Result<bool> {
    if !path.try_exists()? {
        bail!("path doesn't exist");
    }
    if path.is_dir() {
        bail!("path is a directory");
    }

    let metadata = path.metadata()?;
    let permissions = metadata.permissions();
    // check chmod:
    let is_executable = permissions.mode() & 0o111 != 0;

    Ok(is_executable)
}

/// Check if path exists and is executable (bool variant)
fn is_executable(path: &Path) -> bool {
    try_is_executable(path).unwrap_or_default()
}

/// Check if path exists and is writable (Result variant)
async fn try_dir_is_writable(path: &Path) -> anyhow::Result<bool> {
    if !path.is_dir() {
        bail!("path is not a directory");
    }

    // temporarily create file to test write:
    let file_path = path.join("._uvenv_test_file_");
    let file = tokio::fs::File::create(&file_path).await?;
    drop(file);
    tokio::fs::remove_file(&file_path).await?;

    Ok(true)
}

/// Check if path exists and is writable (bool variant)
async fn dir_is_writable(path: &Path) -> bool {
    try_dir_is_writable(path).await.unwrap_or_default()
}

#[derive(
    Debug, Default, serde::Serialize, serde::Deserialize, Hash, Ord, PartialOrd, Eq, PartialEq,
)]
struct PackageVersionInfo {
    current: String,
    latest: Option<Version>,
    is_latest: bool,
}

#[derive(
    Debug, Default, serde::Serialize, serde::Deserialize, Hash, Ord, PartialOrd, Eq, PartialEq,
)]
struct PackageVersions {
    uvenv: PackageVersionInfo,
    uv: PackageVersionInfo,
    patchelf: PackageVersionInfo,
}

#[derive(
    Debug, Default, serde::Serialize, serde::Deserialize, Hash, Ord, PartialOrd, Eq, PartialEq,
)]
struct Python {
    version: String,
    path: String,
    is_executable: bool,
}

#[derive(
    Debug, Default, serde::Serialize, serde::Deserialize, Hash, Ord, PartialOrd, Eq, PartialEq,
)]
struct EnvironmentPaths {
    snap: bool,
    uvenv: String,
    uvenv_ok: bool,
    uv: String,
    uv_ok: bool,
    bin_dir: String,
    bin_dir_ok: bool,
    work_dir: String,
    work_dir_ok: bool,
}

#[derive(
    Debug, Default, serde::Serialize, serde::Deserialize, Hash, Ord, PartialOrd, Eq, PartialEq,
)]
pub struct SelfInfo {
    package_versions: PackageVersions,
    python: Python,
    environment: EnvironmentPaths,
}

async fn uvenv_version_info(_: &Path) -> PackageVersionInfo {
    let latest = get_latest_version("uvenv", true, None).await;

    PackageVersionInfo {
        is_latest: is_latest(CURRENT_UVENV_VERSION, latest.as_ref()),
        current: CURRENT_UVENV_VERSION.to_owned(),
        latest,
    }
}

async fn patchelf_version_info(python_exe: &Path) -> PackageVersionInfo {
    let current = get_package_versions_pip(python_exe, &["patchelf"], "?")
        .await
        .pop()
        .unwrap_or_default();
    let latest = get_latest_version("patchelf", true, None).await;

    PackageVersionInfo {
        is_latest: is_latest(&current, latest.as_ref()),
        current,
        latest,
    }
}

async fn uv_version_info(_: &Path) -> PackageVersionInfo {
    let uv = get_uv_binary().await;
    let output = run_get_output(uv, &["--version"]).await.unwrap_or_default();
    let (_, version) = output.trim().split_once(' ').unwrap_or_default();

    let current = version.to_owned();
    let latest = get_latest_version("uv", true, None).await;

    PackageVersionInfo {
        is_latest: is_latest(&current, latest.as_ref()),
        current,
        latest,
    }
}

async fn package_version_info(python_exe: &Path) -> PackageVersions {
    PackageVersions {
        uvenv: uvenv_version_info(python_exe).await,
        uv: uv_version_info(python_exe).await,
        patchelf: patchelf_version_info(python_exe).await,
    }
}

pub async fn collect_self_info() -> anyhow::Result<SelfInfo> {
    // Find Python and get its version
    let python_exe = find_python().await?;
    let python_version = run_get_output(&python_exe, &["--version"])
        .await
        .unwrap_or_else(|_| "Unknown".to_owned())
        .trim()
        .to_owned();
    let python_is_executable = is_executable(&python_exe);

    // Environment paths
    let me = env::current_exe().unwrap_or_default();
    let uv_path = PathBuf::from(get_uv_binary().await);
    let bin_dir = get_bin_dir();
    let work_dir = get_work_dir();

    let uvenv_ok = is_executable(&me);
    let uv_ok = is_executable(&uv_path);
    let bin_ok = dir_is_writable(&bin_dir).await;
    let work_ok = dir_is_writable(&work_dir).await;

    let info = SelfInfo {
        package_versions: package_version_info(&python_exe).await,
        python: Python {
            version: python_version,
            path: python_exe.to_string(),
            is_executable: python_is_executable,
        },
        environment: EnvironmentPaths {
            snap: cfg!(feature = "snap"),
            uvenv: me.to_string(),
            uvenv_ok,
            uv: uv_path.to_string(),
            uv_ok,
            bin_dir: format!("{}/", bin_dir.as_str()),
            bin_dir_ok: bin_ok,
            work_dir: format!("{}/", work_dir.as_str()),
            work_dir_ok: work_ok,
        },
    };

    Ok(info)
}

pub fn fancy_self_info(info: &SelfInfo) -> anyhow::Result<String> {
    // ANSI escape codes for colors add to the string length.
    // A capacity of 1024 is a safer bet for colored output.
    let mut output = String::with_capacity(1024);

    // Header
    writeln!(output, "{}", "[uvenv Self Information]".bright_magenta())?;

    // Package section
    writeln!(output, "\n{}", "[Package Versions]".bright_blue())?;
    writeln!(
        output,
        "├─ uvenv: {}",
        if info.package_versions.uvenv.is_latest {
            info.package_versions.uvenv.current.green().to_string()
        } else {
            info.package_versions.uvenv.current.red().to_string()
        }
    )?;

    // Display uv version
    let uv_display = if info.package_versions.uv.is_latest {
        info.package_versions.uv.current.green().to_string()
    } else if let Some(latest) = &info.package_versions.uv.latest {
        format!(
            "{} < {}",
            info.package_versions.uv.current.red(),
            latest.yellow() // .to_string() is not strictly needed here if latest is String
        )
    } else {
        info.package_versions.uv.current.yellow().to_string()
    };
    writeln!(output, "├─ uv: {uv_display}")?;

    // Display patchelf version
    let patchelf_display = if info.package_versions.patchelf.is_latest {
        info.package_versions.patchelf.current.green().to_string()
    } else if let Some(latest) = &info.package_versions.patchelf.latest {
        format!(
            "{} < {}",
            info.package_versions.patchelf.current.red(),
            latest.yellow() // .to_string() is not strictly needed here if latest is String
        )
    } else {
        info.package_versions.patchelf.current.yellow().to_string()
    };
    writeln!(output, "└─ patchelf: {patchelf_display}")?;

    // Python info section
    writeln!(output, "\n{}", "[Python]".bright_blue())?;
    writeln!(output, "├─ Version: {}", info.python.version)?;
    writeln!(
        output,
        "└─ Path:    {}",
        red_or_green(&info.python.path, info.python.is_executable)
    )?;

    // Environment section with combined paths
    writeln!(output, "\n{}", "[Paths & Environment]".bright_blue())?;
    if info.environment.snap {
        writeln!(output, "├─ Installation: {}", "snap".cyan())?;
    }

    // Note: The alignment with spaces ("├─ uvenv:        {}") will be preserved.
    // If info.environment.snap is false, the first item in this section will use '├─'.
    // If you want the first item to always be '├─' and subsequent ones '├─' or '└─'
    // dynamically, more complex logic would be needed, or ensure the snap line
    // is always followed by other lines that adjust their prefix.
    // For simplicity, assuming the current structure is desired.
    // If snap is false, the "├─ uvenv:" line will be the first under [Paths & Environment].

    writeln!(
        output,
        "├─ uvenv:        {}",
        red_or_green(&info.environment.uvenv, info.environment.uvenv_ok)
    )?;
    writeln!(
        output,
        "├─ uv:           {}",
        red_or_green(&info.environment.uv, info.environment.uv_ok)
    )?;
    writeln!(
        output,
        "├─ Binaries Dir: {}",
        red_or_green(&info.environment.bin_dir, info.environment.bin_dir_ok)
    )?;
    writeln!(
        output,
        "└─ Working  Dir: {}",
        red_or_green(&info.environment.work_dir, info.environment.work_dir_ok)
    )?;
    writeln!(output)?; // For the final empty line

    Ok(output)
}
pub fn simple_self_info(info: &SelfInfo) -> anyhow::Result<String> {
    // A capacity of 512 is a reasonable starting point.
    // If paths are consistently very long, 768 or 1024 might be better.
    let mut output = String::with_capacity(512);

    // Header
    writeln!(output, "[uvenv Self Information]")?; // ? is fine here as writing to String shouldn't fail

    // Package section
    writeln!(output, "\n[Package Versions]")?;
    writeln!(
        output,
        "- uvenv: {} {}",
        info.package_versions.uvenv.current,
        if info.package_versions.uvenv.is_latest {
            "(latest)"
        } else {
            "(outdated)"
        }
    )?;

    // Display uv version
    let uv_status = if info.package_versions.uv.is_latest {
        "(latest)".to_owned()
    } else if let Some(latest) = &info.package_versions.uv.latest {
        format!("(latest: {latest})")
    } else {
        "(status unknown)".to_owned()
    };
    writeln!(
        output,
        "- uv: {} {}",
        info.package_versions.uv.current, uv_status
    )?;

    // Display patchelf version
    let patchelf_status = if info.package_versions.patchelf.is_latest {
        "(latest)".to_owned()
    } else if let Some(latest) = &info.package_versions.patchelf.latest {
        format!("(latest: {latest})")
    } else {
        "(status unknown)".to_owned()
    };
    writeln!(
        output,
        "- patchelf: {} {}",
        info.package_versions.patchelf.current, patchelf_status
    )?;

    // Python info section
    writeln!(output, "\n[Python]")?;
    writeln!(output, "- Version: {}", info.python.version)?;
    writeln!(
        output,
        "- Path: {} {}",
        info.python.path,
        if info.python.is_executable {
            "(OK)"
        } else {
            "(NOT EXECUTABLE)"
        }
    )?;

    // Environment section
    writeln!(output, "\n[Paths & Environment]")?;
    if info.environment.snap {
        writeln!(output, "- Installation: snap")?;
    }
    writeln!(
        output,
        "- uvenv: {} {}",
        info.environment.uvenv,
        if info.environment.uvenv_ok {
            "(OK)"
        } else {
            "(NOT EXECUTABLE)"
        }
    )?;
    writeln!(
        output,
        "- uv: {} {}",
        info.environment.uv,
        if info.environment.uv_ok {
            "(OK)"
        } else {
            "(NOT EXECUTABLE)"
        }
    )?;
    writeln!(
        output,
        "- Binaries Dir: {} {}",
        info.environment.bin_dir,
        if info.environment.bin_dir_ok {
            "(OK)"
        } else {
            "(NOT WRITABLE)"
        }
    )?;
    writeln!(
        output,
        "- Working Dir: {} {}",
        info.environment.work_dir,
        if info.environment.work_dir_ok {
            "(OK)"
        } else {
            "(NOT WRITABLE)"
        }
    )?;
    writeln!(output)?; // For the final empty line

    Ok(output)
}

/// Basic variant for use by `self_version`
pub async fn self_info() -> anyhow::Result<i32> {
    let info = collect_self_info().await?;
    print!("{}", simple_self_info(&info)?);

    Ok(0)
}

impl Process for SelfInfoOptions {
    async fn process(self) -> anyhow::Result<i32> {
        let info = collect_self_info().await?;
        let to_print = if self.simple {
            simple_self_info(&info)?
        } else if self.json {
            serde_json::to_string_pretty(&info)?
        } else if self.toml {
            toml::to_string_pretty(&info)?
        } else if self.fancy {
            fancy_self_info(&info)?
        } else {
            bail!("Unexpected format.")
        };

        print!("{to_print}");

        Ok(0)
    }
}
