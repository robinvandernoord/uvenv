use crate::cli::{Process, ReinstallAllOptions};
use crate::commands::list::list_packages;
use crate::commands::reinstall::reinstall;
use crate::metadata::LoadMetadataConfig;
use anyhow::{anyhow, Context};

pub async fn reinstall_all(
    python: Option<&String>,
    force: bool,
    without_injected: bool,
    no_cache: bool,
    editable: bool,
    venv_names: &[String],
) -> anyhow::Result<()> {
    let mut all_ok = true;
    // only used if not all_ok, but already created for chaining:
    let mut err_result = Err(anyhow!("-> Failed reinstall-all."));

    for meta in list_packages(&LoadMetadataConfig::none(), Some(venv_names), None).await? {
        match reinstall(
            &meta.name,
            python,
            force,
            !without_injected,
            no_cache,
            editable,
        )
        .await
        {
            Ok(msg) => {
                println!("{msg}");
            },
            Err(msg) => {
                err_result = err_result.with_context(|| msg);
                // eprintln!("{}", msg.red());
                all_ok = false;
            },
        }
    }
    if all_ok {
        Ok(())
    } else {
        err_result.with_context(|| "⚠️ Not all packages were properly reinstalled!")
    }
}

impl Process for ReinstallAllOptions {
    async fn process(self) -> anyhow::Result<i32> {
        match reinstall_all(
            self.python.as_ref(),
            self.force,
            self.without_injected,
            self.no_cache,
            self.editable,
            &self.venv_names,
        )
        .await
        {
            Ok(()) => Ok(0),
            Err(msg) => Err(msg),
        }
    }
}
