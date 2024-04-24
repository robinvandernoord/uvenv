use crate::pip::parse_requirement;
use crate::uv::Helpers;
use owo_colors::OwoColorize;

use crate::cli::{Process, UninstallOptions};
use crate::metadata::{venv_path, Metadata};
use crate::symlinks::{find_symlinks, remove_symlink, remove_symlinks};
use crate::venv::{activate_venv, remove_venv};

pub async fn uninstall_package(
    package_name: &str,
    force: bool,
) -> Result<String, String> {
    let (requirement, _) = parse_requirement(package_name).await?;
    let requirement_name = requirement.name.to_string();

    let venv_dir = venv_path(&requirement_name);

    if !venv_dir.exists() {
        return if !force {
            Err(format!("No virtualenv for '{}', stopping.\nUse '{}' to remove an executable with that name anyway.",
                        &requirement_name.green(), "--force".green()))
        } else {
            remove_symlink(&requirement_name).await?;
            Err(format!(
                "{}: No virtualenv for '{}'.",
                "Warning".yellow(),
                &requirement_name.green()
            ))
        };
    }

    let venv = activate_venv(&venv_dir).await?;

    let metadata = Metadata::for_requirement(&requirement, false).await;

    // symlinks = find_symlinks(package_name, venv_path) or [package_name]
    let symlinks = find_symlinks(&requirement, &metadata.installed_version, &venv).await;

    remove_symlinks(symlinks).await?;

    remove_venv(&venv.to_path_buf()).await?;

    let version_msg = if !metadata.installed_version.is_empty() {
        format!(" ({})", metadata.installed_version.cyan())
    } else {
        String::new()
    };

    let msg = format!("🗑️  {}{} removed!", package_name, version_msg);

    Ok(msg)
}

impl Process for UninstallOptions {
    async fn process(self) -> Result<i32, String> {
        match uninstall_package(&self.package_name, self.force).await {
            Ok(msg) => {
                println!("{}", msg);
                Ok(0)
            },
            Err(msg) => Err(msg),
        }
    }
}
