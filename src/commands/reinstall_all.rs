use crate::cli::{Process, ReinstallAllOptions};
use crate::commands::list::list_packages;
use crate::commands::reinstall::reinstall;
use owo_colors::OwoColorize;

pub async fn reinstall_all(
    python: Option<&String>,
    force: bool,
    without_injected: bool,
    no_cache: bool,
    editable: bool,
) -> Result<(), String> {
    let mut all_ok = true;

    for meta in list_packages().await? {
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
        )
        .await
        {
            Ok(_) => Ok(0),
            Err(msg) => Err(msg),
        }
    }
}
