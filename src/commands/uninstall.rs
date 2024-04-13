use crate::helpers::ResultToString;
use crate::uv::Helpers;
use owo_colors::OwoColorize;
use pep508_rs::Requirement;
use std::str::FromStr;

use crate::metadata::Metadata;
use crate::symlinks::{find_symlinks, remove_symlink, remove_symlinks};
use crate::venv::{activate_venv, remove_venv};
use crate::{
    cli::{Process, UninstallOptions},
    metadata::get_venv_dir,
};

pub async fn uninstall_package(package_name: &str, force: bool) -> Result<String, String> {
    let requirement = Requirement::from_str(package_name).map_err_to_string()?;
    let requirement_name = requirement.name.to_string();

    let workdir = get_venv_dir();
    let venv_dir = workdir.join(&requirement_name);

    if !venv_dir.exists() {
        if !force {
            return Err(format!("No virtualenv for '{}', stopping.\nUse '{}' to remove an executable with that name anyway.", 
            &requirement_name.green(), "--force".green()));
        } else {
            remove_symlink(&requirement_name).await?;
            return Err(format!(
                "No virtualenv for '{}', stopping.",
                &requirement_name.green()
            ));
        }
    }

    let venv = activate_venv(&venv_dir).await?;

    let metadata = match Metadata::for_dir(&venv_dir, false).await {
        Some(m) => m,
        None => Metadata::find(&requirement),
    };

    // symlinks = find_symlinks(package_name, venv_path) or [package_name]
    let symlinks = find_symlinks(&metadata, &venv).await;

    remove_symlinks(symlinks).await?;

    remove_venv(&venv.to_path_buf()).await?;

    let version_msg = if metadata.installed_version != "" {
        format!(" ({})", metadata.installed_version.cyan())
    } else {
        String::new()
    };

    let msg = format!("🗑️  {}{} removed!", package_name, version_msg);

    return Ok(msg);
}

impl Process for UninstallOptions {
    async fn process(self) -> Result<u32, String> {
        match uninstall_package(&self.package_name, self.force).await {
            Ok(msg) => {
                println!("{}", msg);
                return Ok(0);
            }
            Err(msg) => return Err(msg),
        }
    }
}
