use std::path::Path;

use pep508_rs::Requirement;
use uv_interpreter::PythonEnvironment;

use crate::metadata::get_bin_dir;
use configparser::ini::Ini;

pub async fn console_scripts(entry_points_path: &str) -> Result<Vec<String>, String> {
    let Ok(ini) = tokio::fs::read_to_string(entry_points_path).await else {
        return Ok(Vec::new()); // file missing = empty list
    };

    let entry_points_mapping = Ini::new_cs()
        .read(ini)
        .map_err(|err| format!("entry_points.txt is invalid: {err}"))?;

    let Some(console_scripts) = entry_points_mapping.get("console_scripts") else {
        return Ok(Vec::new());
    };

    // extract script keys
    return Ok(console_scripts.keys().map(|k| k.to_string()).collect());
}

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

    let entrypoints_ini = venv
        .interpreter()
        .purelib()
        .join(dist_info_fname)
        .join("entry_points.txt");
    let entrypoints_path = entrypoints_ini.to_str().unwrap_or_default();
    let scripts = console_scripts(entrypoints_path).await.unwrap_or_default();

    if !scripts.is_empty() {
        scripts
    } else {
        // no scripts found, use requirement name as fallback (e.g. for `uv`)
        vec![requirement.name.to_string()]
    }
}

pub async fn create_symlink(
    symlink: &str,
    venv: &Path,
    force: bool,
    binaries: &[&str],
) -> Result<bool, String> {
    let bin_dir = get_bin_dir();

    if !binaries.is_empty() && !binaries.contains(&symlink) {
        return Ok(false);
    }

    let target_path = bin_dir.join(symlink);

    if target_path.exists() {
        if !force {
            return Err(format!(
                "Script {} already exists in {:?}. Use --force to ignore this warning.",
                symlink, bin_dir
            ));
        }
        tokio::fs::remove_file(&target_path)
            .await
            .map_err(|_| format!("Failed to create symlink {:?}", &target_path))?;
    }

    let symlink_path = venv.join("bin").join(symlink);
    if !symlink_path.exists() {
        return Err(format!(
            "Could not symlink {:?} because the script didn't exist.",
            symlink_path
        ));
    }

    tokio::fs::symlink(&symlink_path, &target_path)
        .await
        .map_err(|_| format!("Failed to create symlink {:?}", &target_path))?;

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
        .map_or(false, |link| link.starts_with(target_path))
}

pub async fn check_symlink(
    symlink: &str,
    target_path: &Path,
) -> bool {
    let symlink_path = get_bin_dir().join(symlink);

    is_symlink(&symlink_path) && points_to(&symlink_path, target_path)
}

pub async fn remove_symlink(symlink: &str) -> Result<(), String> {
    let bin_dir = get_bin_dir();
    let target_path = bin_dir.join(symlink);

    if is_symlink(&target_path) {
        tokio::fs::remove_file(&target_path)
            .await
            .map_err(|_| format!("Failed to remove symlink {:?}", &target_path))?;
    };

    Ok(())
}

pub async fn remove_symlinks(symlinks: Vec<String>) -> Result<(), String> {
    for symlink in symlinks {
        remove_symlink(&symlink).await?;
    }

    Ok(())
}
