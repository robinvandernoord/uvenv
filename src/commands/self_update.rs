use std::path::Path;

use owo_colors::OwoColorize;
use regex::Regex;

use crate::animate::{show_loading_indicator, AnimationSettings};
use crate::cli::{Process, SelfUpdateOptions};
use crate::cmd::{find_sibling, run};
use crate::pip::pip_freeze;

fn extract_version(
    freeze_output: &str,
    needle: &str,
) -> Option<String> {
    // (?m) for multi-line
    let Ok(re) = Regex::new(&format!("(?m)^({needle}==|{needle} @)(.+)")) else {
        return None;
    };

    // for version in re.captures_iter(freeze_output) {
    if let Some(version) = re.captures_iter(freeze_output).next() {
        let (_, [_, version]) = version.extract();
        // group 1 is {package}==
        // group 2 is the version (or path/uri)
        return Some(version.to_string());
    }

    None
}

#[allow(dead_code)]
pub async fn get_package_version(
    python: &Path,
    package: &str,
) -> Option<String> {
    let output = pip_freeze(python).await.unwrap_or_default();

    extract_version(&output, package)
}

pub async fn get_package_versions<S: AsRef<str>>(
    python: &Path,
    packages: &[S],
) -> Vec<String> {
    let output = pip_freeze(python).await.unwrap_or_default();

    packages
        .iter()
        .map(|k| extract_version(&output, k.as_ref()).unwrap_or_default())
        .collect()
}

pub async fn self_update(with_uv: bool) -> Result<i32, String> {
    let Some(exe) = find_sibling("python").await else {
        return Err(format!(
            "Python could not be found! Is `{}` installed globally (without a venv)?",
            "uvx".green()
        ));
    };

    // todo: with 'uv' instead of pip later?
    let mut args = vec!["-m", "pip", "install", "--no-cache-dir", "--upgrade", "uvx"];

    let mut to_track = vec!["uvx"];
    let mut msg = String::from("uvx");
    if with_uv {
        args.push("uv");
        to_track.push("uv");
        msg.push_str(" and uv")
    }

    let old = get_package_versions(&exe, &to_track).await;

    let exe_str = exe.to_str().unwrap_or_default();
    let promise = run(&exe_str, args, None);

    show_loading_indicator(
        promise,
        format!("updating {}", msg),
        AnimationSettings::default(),
    )
    .await?;

    let new = get_package_versions(&exe, &to_track).await;

    for (versions, package) in new.iter().zip(old.iter()).zip(to_track.iter()) {
        let (after, before) = versions;
        if before == after {
            println!(
                "🌟 '{}' not updated (version: {})",
                package.blue(),
                before.green()
            )
        } else {
            println!(
                "🚀 '{}' updated from {} to {}",
                package.blue(),
                before.red(),
                after.green(),
            )
        }
    }

    Ok(0)
}

impl Process for SelfUpdateOptions {
    async fn process(self) -> Result<i32, String> {
        self_update(!self.without_uv).await
    }
}
