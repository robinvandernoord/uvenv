use owo_colors::OwoColorize;

use crate::cli::{Process, UpgradeAllOptions};
use crate::commands::list::list_packages;
use crate::commands::upgrade::upgrade_package;

pub async fn upgrade_all(
    force: bool,
    no_cache: bool,
    skip_injected: bool,
) -> Result<(), String> {
    let mut all_ok = true;

    for meta in list_packages().await? {
        match upgrade_package(&meta.name, force, no_cache, skip_injected).await {
            Ok(msg) => {
                println!("{}", msg)
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
    async fn process(self) -> Result<i32, String> {
        match upgrade_all(self.force, self.no_cache, self.skip_injected).await {
            Ok(_) => Ok(0),
            Err(msg) => Err(msg),
        }
    }
}
