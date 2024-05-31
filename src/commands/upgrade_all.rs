use crate::cli::{Process, UpgradeAllOptions};
use crate::commands::list::list_packages;
use crate::commands::upgrade::upgrade_package_owned;
use crate::metadata::LoadMetadataConfig;
use crate::promises::handle_promises;

pub async fn upgrade_all(
    force: bool,
    no_cache: bool,
    skip_injected: bool,
    venv_names: &[String],
) -> Result<(), String> {
    let mut promises = vec![];

    for meta in list_packages(&LoadMetadataConfig::none(), Some(venv_names)).await? {
        promises.push(upgrade_package_owned(
            meta.name,
            force,
            no_cache,
            skip_injected,
        ));
    }

    let promise_len = promises.len();
    let results = handle_promises(promises).await;
    let all_ok = promise_len == results.len();

    for msg in results {
        eprintln!("{msg}");
    }
    if all_ok {
        Ok(())
    } else {
        Err(String::from("⚠️ Not all packages were properly upgraded!"))
    }
}

impl Process for UpgradeAllOptions {
    async fn process(self) -> Result<i32, String> {
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
