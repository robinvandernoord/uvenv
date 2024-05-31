use crate::cli::{Process, ReinstallAllOptions};
use crate::commands::list::list_packages;
use crate::commands::reinstall::reinstall_owned;
use crate::metadata::LoadMetadataConfig;
use crate::promises::handle_promises;

pub async fn reinstall_all(
    python: Option<&String>,
    force: bool,
    without_injected: bool,
    no_cache: bool,
    editable: bool,
    venv_names: &[String],
) -> Result<(), String> {
    // pre: 158 ms

    let mut promises = vec![];

    for meta in list_packages(&LoadMetadataConfig::none(), Some(venv_names)).await? {
        promises.push(reinstall_owned(
            meta.name,
            python,
            force,
            !without_injected,
            no_cache,
            editable,
        ));
    }

    let promise_len = promises.len();

    let results = handle_promises(promises).await;

    let all_ok = results.len() == promise_len;

    if all_ok {
        if venv_names.is_empty() {
            eprintln!("📦 All packages reinstalled.");
        } else {
            for msg in results {
                eprintln!("{msg}");
            }
        }

        Ok(())
    } else {
        Err(String::from(
            "⚠️ Not all packages were properly reinstalled!",
        ))
    }
}

impl Process for ReinstallAllOptions {
    async fn process(self) -> Result<i32, String> {
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
