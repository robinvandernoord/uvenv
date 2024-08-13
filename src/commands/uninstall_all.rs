use crate::cli::{Process, UninstallAllOptions};
use crate::commands::list::list_packages;
use crate::commands::uninstall::uninstall_package;
use crate::metadata::LoadMetadataConfig;
use anyhow::{anyhow, Context};

pub async fn uninstall_all(
    force: bool,
    venv_names: &[String],
) -> anyhow::Result<()> {
    let mut all_ok = true;
    let mut err_result = Err(anyhow!("-> Failed uninstall-all."));

    for meta in list_packages(&LoadMetadataConfig::none(), Some(venv_names), None).await? {
        match uninstall_package(&meta.name, force).await {
            Ok(msg) => {
                println!("{msg}");
            },
            Err(msg) => {
                // eprintln!("{}", msg.red());
                err_result = err_result.with_context(|| msg);
                all_ok = false;
            },
        }
    }

    if all_ok {
        Ok(())
    } else {
        err_result.with_context(|| "⚠️ Not all packages were properly uninstalled!")
    }
}

impl Process for UninstallAllOptions {
    async fn process(self) -> anyhow::Result<i32> {
        match uninstall_all(self.force, &self.venv_names).await {
            Ok(()) => Ok(0),
            Err(msg) => Err(msg),
        }
    }
}
