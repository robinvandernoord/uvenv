use std::collections::BTreeMap;

use futures::future;
use owo_colors::OwoColorize;
use pep440_rs::Version;

use crate::cli::{Process, SelfVersionOptions};
use crate::cmd::run_get_output;
use crate::commands::self_update::{find_python, get_package_versions};
use crate::helpers::{flatten_option_ref, PathToString};
use crate::pypi::get_latest_version;

async fn get_latest_versions(package_names: Vec<&str>) -> BTreeMap<String, Option<Version>> {
    let promises: Vec<_> = package_names
        .iter()
        .map(|it| get_latest_version(it, true, None))
        .collect();
    let resolved = future::join_all(promises).await;

    let mut result = BTreeMap::new();
    for (package, version) in package_names.into_iter().zip(resolved.into_iter()) {
        result.insert(package.to_string(), version);
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
pub fn _compare_versions(
    current: &str,
    latest: &str,
) -> bool {
    current.ge(latest)
}

pub fn is_latest(
    current: &str,
    latest: Option<&Version>,
) -> bool {
    let Some(version) = latest else { return false };

    _compare_versions(current, &version.to_string())
}

pub const fn uvenv_version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}

impl Process for SelfVersionOptions {
    async fn process(self) -> anyhow::Result<i32> {
        let exe = find_python().await?;
        let python_version = run_get_output(&exe, &["--version"]).await.ok();

        let latest = get_latest_versions(vec!["uvenv", "uv", "patchelf"]).await;

        // uvenv version comes from Cargo.toml
        let version = uvenv_version();
        let to_track = ["uv", "patchelf"]; // + Python version
        let versions = get_package_versions(&exe, &to_track, "?").await;

        let uvenv_is_latest = is_latest(version, flatten_option_ref(latest.get("uvenv")));
        println!("- uvenv: {}", red_or_green(version, uvenv_is_latest));

        for (package, version) in to_track.into_iter().zip(versions.iter()) {
            let pkg_is_latest = is_latest(version, flatten_option_ref(latest.get(package)));
            println!("  - {}: {}", package, red_or_green(version, pkg_is_latest));
        }

        if let Some(py_version) = python_version {
            eprintln!("\n{} ({})", py_version.trim(), exe.to_string().trim());
        }

        Ok(0)
    }
}
