use std::collections::BTreeMap;

use futures::future;
use owo_colors::OwoColorize;
use uv_pep440::Version;

use crate::cli::{Process, SelfVersionOptions};
use crate::cmd::run_get_output;
use crate::commands::self_update::{find_python, get_package_versions_pip};
use crate::helpers::{flatten_option_ref, PathToString};
use crate::pypi::get_latest_version;

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
        let versions = get_package_versions_pip(&exe, &to_track, "?").await;

        let uvenv_is_latest = is_latest(version, flatten_option_ref(latest.get("uvenv")));
        println!("- uvenv: {}", red_or_green(version, uvenv_is_latest));

        for (package_name, package_version) in to_track.into_iter().zip(versions.iter()) {
            let maybe_latest_version = flatten_option_ref(latest.get(package_name));

            if is_latest(package_version, maybe_latest_version) {
                println!("  - {}: {}", package_name, package_version.green());
            } else if let Some(latest_version) = maybe_latest_version {
                // show installed in red and latest in yellow:
                println!(
                    "  - {}: {} < {}",
                    package_name,
                    package_version.red(),
                    latest_version.yellow()
                );
            } else {
                // latest unknown - make it yellow
                println!("  - {}: {}", package_name, package_version.yellow());
            }
        }

        if let Some(py_version) = python_version {
            eprintln!("\n{} ({})", py_version.trim(), exe.to_string().trim());
        }

        Ok(0)
    }
}
