use crate::cli::{Process, UpgradeAllOptions};
use crate::commands::list::list_packages;
use crate::commands::upgrade::upgrade_package;
use crate::metadata::LoadMetadataConfig;
use anyhow::{anyhow, Context};

pub async fn upgrade_all(
    force: bool,
    no_cache: bool,
    skip_injected: bool,
    venv_names: &[String],
) -> anyhow::Result<()> {
    let mut all_ok = true;
    let mut err_result = Err(anyhow!("-> Failed upgrade-all."));

    for meta in list_packages(&LoadMetadataConfig::none(), Some(venv_names), None).await? {
        match upgrade_package(&meta.name, force, no_cache, skip_injected).await {
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
        err_result.with_context(|| "⚠️ Not all packages were properly upgraded!")
    }
}

impl Process for UpgradeAllOptions {
    async fn process(self) -> anyhow::Result<i32> {
        match upgrade_all(
            self.force,
            self.no_cache,
            self.skip_injected,
            &self.venv_names,
        )
        .await
        {
            Ok(()) => Ok(0),
            Err(msg) => Err(msg),
        }
    }
}
