use crate::cli::{Process, UpgradeAllOptions};
use crate::commands::list::list_packages;
use crate::commands::upgrade::upgrade_package;
use crate::metadata::LoadMetadataConfig;
use anyhow::anyhow;
use owo_colors::OwoColorize;

pub async fn upgrade_all(
    force: bool,
    no_cache: bool,
    skip_injected: bool,
    venv_names: &[String],
) -> Result<(), String> {
    let mut all_ok = true;
    for meta in list_packages(&LoadMetadataConfig::none(), Some(venv_names)).await? {
        match upgrade_package(&meta.name, force, no_cache, skip_injected).await {
            Ok(msg) => {
                println!("{msg}");
            },
            Err(msg) => {
                eprintln!("{}", msg.red());
                all_ok = false;
            },
        }
    }

    if all_ok {
        Ok(())
    } else {
        Err(String::from("⚠️ Not all packages were properly upgraded!"))
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
            Err(msg) => Err(anyhow!(msg)),
        }
    }
}
