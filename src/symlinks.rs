use anyhow::{Context, anyhow, bail};
use std::path::{Path, PathBuf};

use uv_normalize::PackageName;
use uv_pep508::Requirement;
use uv_python::PythonEnvironment;

use crate::helpers::PathAsStr;
use crate::metadata::ensure_bin_dir;
use configparser::ini::Ini;
use uv_distribution_types::Name;
use uv_installer::SitePackages;
use uv_tool::entrypoint_paths;

pub async fn console_scripts(entry_points_path: &str) -> anyhow::Result<Vec<String>> {
    let Ok(ini) = tokio::fs::read_to_string(entry_points_path).await else {
        return Ok(Vec::new()); // file missing = empty list
    };

    let entry_points_mapping = Ini::new_cs()
        .read(ini)
        .map_err(|err| anyhow!("entry_points.txt is invalid: {err}"))?;

    let Some(console_scripts) = entry_points_mapping.get("console_scripts") else {
        return Ok(Vec::new());
    };

    // extract script keys
    Ok(console_scripts.keys().map(ToString::to_string).collect())
}

/// Source: `https://github.com/astral-sh/uv/blob/ee2bdc21fab077aaef17c94242b4cf6a10c013e1/crates/uv/src/commands/tool/run.rs#L281`
fn get_entrypoints(
    from: &PackageName,
    site_packages: &SitePackages,
) -> anyhow::Result<Vec<(String, PathBuf)>> {
    let installed = site_packages.get_packages(from);
    let Some(installed_dist) = installed.first().copied() else {
        bail!("Expected at least one requirement")
    };

    Ok(entrypoint_paths(
        site_packages,
        installed_dist.name(),
        installed_dist.version(),
    )?)
}

fn find_symlinks_uv(
    requirement: &Requirement,
    venv: &PythonEnvironment,
) -> anyhow::Result<Vec<String>> {
    let site_packages = SitePackages::from_environment(venv)?;
    let entrypoints = get_entrypoints(&requirement.name, &site_packages)?;

    Ok(entrypoints
        .iter()
        .map(|(name, _path)| name.to_owned())
        .collect())
}

#[expect(clippy::shadow_unrelated, reason = "Both `scripts` are related.")]
pub async fn find_symlinks(
    requirement: &Requirement,
    installed_version: &str,
    venv: &PythonEnvironment,
) -> Vec<String> {
    let dist_info_fname = format!(
        "{}-{}.dist-info",
        requirement.name.as_dist_info_name(),
        installed_version
    );

    // uv:
    let scripts = find_symlinks_uv(requirement, venv).unwrap_or_default();

    if !scripts.is_empty() {
        return scripts;
    }

    // fallback:

    let entrypoints_ini = venv
        .interpreter()
        .purelib()
        .join(dist_info_fname)
        .join("entry_points.txt");
    let entrypoints_path = entrypoints_ini.as_str();

    let scripts = console_scripts(entrypoints_path).await.unwrap_or_default();

    if scripts.is_empty() {
        // no scripts found, use requirement name as fallback (e.g. for `uv`)
        vec![requirement.name.to_string()]
    } else {
        scripts
    }
}

pub async fn create_symlink(
    symlink: &str,
    venv: &Path,
    force: bool,
    binaries: &[&str],
) -> anyhow::Result<bool> {
    let bin_dir = ensure_bin_dir().await;

    if !binaries.is_empty() && !binaries.contains(&symlink) {
        return Ok(false);
    }

    let target_path = bin_dir.join(symlink);

    if target_path.exists() {
        if !force {
            bail!(
                "Script {symlink} already exists in {}. Use --force to ignore this warning.",
                bin_dir.display()
            )
        }

        tokio::fs::remove_file(&target_path)
            .await
            .with_context(|| format!("Failed to create symlink {}", target_path.display()))?;
    }

    let symlink_path = venv.join("bin").join(symlink);
    if !symlink_path.exists() {
        bail!(
            "Could not symlink {} because the script didn't exist.",
            symlink_path.display()
        );
    }

    tokio::fs::symlink(&symlink_path, &target_path)
        .await
        .with_context(|| format!("Failed to create symlink {}", target_path.display()))?;

    Ok(true)
}

pub fn is_symlink(symlink_path: &Path) -> bool {
    symlink_path
        .symlink_metadata()
        .map(|metadata| metadata.file_type().is_symlink())
        .unwrap_or(false)
}

pub fn points_to(
    symlink_path: &Path,
    target_path: &Path,
) -> bool {
    symlink_path
        .read_link()
        .ok()
        .is_some_and(|link| link.starts_with(target_path))
}

pub async fn check_symlink(
    symlink: &str,
    target_path: &Path,
) -> bool {
    let symlink_path = ensure_bin_dir().await.join(symlink);

    is_symlink(&symlink_path) && points_to(&symlink_path, target_path)
}

pub async fn remove_symlink(symlink: &str) -> anyhow::Result<()> {
    let bin_dir = ensure_bin_dir().await;
    let target_path = bin_dir.join(symlink);

    if is_symlink(&target_path) {
        tokio::fs::remove_file(&target_path).await?;
    }

    Ok(())
}

pub async fn remove_symlinks(symlinks: &[String]) -> anyhow::Result<()> {
    for symlink in symlinks {
        remove_symlink(symlink).await?;
    }

    Ok(())
}
