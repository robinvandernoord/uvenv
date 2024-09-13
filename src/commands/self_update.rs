use anyhow::{anyhow, bail, Context};
use std::env;
use std::path::{Path, PathBuf};

use crate::animate::{show_loading_indicator, AnimationSettings};
use crate::cli::{Process, SelfUpdateOptions};
use crate::cmd::{find_sibling, run};
use crate::helpers::PathAsStr;
use crate::pip::pip_freeze;
use crate::uv::{system_environment, uv_freeze, PythonSpecifier};
use owo_colors::OwoColorize;
use regex::Regex;

fn extract_version(
    freeze_output: &str,
    needle: &str,
) -> Option<String> {
    // (?m) for multi-line
    let Ok(re) = Regex::new(&format!("(?m)^({needle}==|{needle} @)(.+)")) else {
        return None;
    };

    // for version in re.captures_iter(freeze_output) {
    if let Some(version_cap) = re.captures_iter(freeze_output).next() {
        let (_, [_, version]) = version_cap.extract();
        // group 1 is {package}==
        // group 2 is the version (or path/uri)
        return Some(version.to_owned());
    }

    None
}

pub async fn get_package_versions_uv<S: AsRef<str>>(
    python: &Path,
    packages: &[S],
    default: &str,
) -> Vec<String> {
    let output = uv_freeze(PythonSpecifier::Path(python))
        .await
        .unwrap_or_default();

    packages
        .iter()
        .map(|pkg_name| {
            extract_version(&output, pkg_name.as_ref()).unwrap_or_else(|| default.to_owned())
        })
        .collect()
}

pub async fn get_package_versions_pip<S: AsRef<str>>(
    python: &Path,
    packages: &[S],
    default: &str,
) -> Vec<String> {
    let output = pip_freeze(python).await.unwrap_or_default();

    packages
        .iter()
        .map(|pkg_name| {
            extract_version(&output, pkg_name.as_ref()).unwrap_or_else(|| default.to_owned())
        })
        .collect()
}

pub fn find_global_python() -> anyhow::Result<PathBuf> {
    let maybe_environ = system_environment().ok();

    if let Some(environ) = maybe_environ {
        return Ok(environ.python_executable().to_path_buf());
    }

    // naive fallback:

    let fallback = PathBuf::from("/usr/bin/python3");
    if fallback.exists() {
        Ok(fallback)
    } else {
        bail!(
            "Python could not be found! Is `{}` installed globally (without a venv)?",
            "uvenv".green()
        )
    }
}

pub async fn find_python() -> anyhow::Result<PathBuf> {
    find_sibling("python")
        .await
        .map_or_else(find_global_python, Ok)
}

pub async fn self_update_via_pip(
    with_uv: bool,
    with_patchelf: bool,
) -> anyhow::Result<i32> {
    // fallback for self_update_via_uv

    let exe = find_python().await?;

    let mut args = vec![
        "-m",
        "pip",
        "install",
        "--no-cache-dir",
        // "--break-system-packages",
        "--upgrade",
        "uvenv",
    ];
    env::set_var("PIP_BREAK_SYSTEM_PACKAGES", "1");

    let mut to_track = vec!["uvenv"];
    let mut msg = String::from("uvenv");
    if with_uv {
        args.push("uv");
        to_track.push("uv");
        msg.push_str(" and uv");
    }

    if with_patchelf {
        args.push("patchelf");
        to_track.push("patchelf");
        msg.push_str(" and patchelf");
    }

    let old = get_package_versions_pip(&exe, &to_track, "?").await;

    let exe_str = exe.as_str();
    let promise = run(&exe_str, &args, None);

    show_loading_indicator(
        promise,
        format!("updating {msg} through pip"),
        AnimationSettings::default(),
    )
    .await?;

    let new = get_package_versions_pip(&exe, &to_track, "?").await;

    handle_self_update_result(&to_track, &old, &new);

    Ok(0)
}

/// currently dead but kept here for documentation
/// (+ so functions like `uv_freeze` don't cound as dead code, it could be useful later.)
#[expect(
    dead_code,
    reason = "It took so much work to make this kind of work I don't want to remove it now."
)]
pub async fn self_update_via_uv(
    with_uv: bool,
    with_patchelf: bool,
) -> anyhow::Result<i32> {
    let python_exe = find_python().await?;

    let uv = find_sibling("uv")
        .await
        .ok_or_else(|| anyhow!("Could not find uv!"))?;

    let mut args = vec![
        "pip",
        "install",
        "--no-cache-dir",
        "--python",
        &python_exe.as_str(),
        "--upgrade",
        "uvenv",
    ];
    env::set_var("UV_BREAK_SYSTEM_PACKAGES", "1");

    let mut to_track = vec!["uvenv"];
    let mut msg = String::from("uvenv");
    if with_uv {
        args.push("uv");
        to_track.push("uv");
        msg.push_str(" and uv");
    }

    if with_patchelf {
        args.push("patchelf");
        to_track.push("patchelf");
        msg.push_str(" and patchelf");
    }

    let old = get_package_versions_uv(&python_exe, &to_track, "?").await;

    let promise = run(&uv, &args, None);

    show_loading_indicator(
        promise,
        format!("updating {msg} through uv"),
        AnimationSettings::default(),
    )
    .await?;

    let new = get_package_versions_uv(&python_exe, &to_track, "?").await;

    handle_self_update_result(&to_track, &old, &new);

    Ok(0)
}

fn handle_self_update_result(
    to_track: &[&str],
    old: &[String],
    new: &[String],
) {
    for (versions, package) in new.iter().zip(old.iter()).zip(to_track.iter()) {
        let (after, before) = versions;
        if before == after {
            println!(
                "🌟 '{}' not updated (version: {})",
                package.blue(),
                before.green()
            );
        } else {
            println!(
                "🚀 '{}' updated from {} to {}",
                package.blue(),
                before.red(),
                after.green(),
            );
        }
    }
}

pub async fn self_update(
    with_uv: bool,
    with_patchelf: bool,
) -> anyhow::Result<i32> {
    // note: uv doesn't work for --user (only --system, which is not allowed on ubuntu 24.04)
    // so just using pip is most stable (although a bit slower):
    self_update_via_pip(with_uv, with_patchelf).await
}

impl Process for SelfUpdateOptions {
    async fn process(self) -> anyhow::Result<i32> {
        self_update(!self.without_uv, !self.without_patchelf)
            .await
            .with_context(|| "Something went wrong trying to update 'uvenv';")
    }
}
