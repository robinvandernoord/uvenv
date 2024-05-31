use crate::cli::{Process, UninstallAllOptions};
use crate::commands::list::list_packages;
use crate::commands::uninstall::uninstall_package_owned;
use crate::metadata::LoadMetadataConfig;
use crate::promises::handle_promises;

pub async fn uninstall_all(
    force: bool,
    venv_names: &[String],
) -> Result<(), String> {
    let mut promises = vec![];

    for meta in list_packages(&LoadMetadataConfig::none(), Some(venv_names)).await? {
        promises.push(uninstall_package_owned(meta.name, force));
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
        Err(String::from(
            "⚠️ Not all packages were properly uninstalled!",
        ))
    }
}

impl Process for UninstallAllOptions {
    async fn process(self) -> Result<i32, String> {
        match uninstall_all(self.force, &self.venv_names).await {
            Ok(()) => Ok(0),
            Err(msg) => Err(msg),
        }
    }
}
