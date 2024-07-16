use crate::cli::{Process, SelfMigrateOptions};
use crate::commands::reinstall_all::reinstall_all;
use crate::metadata::get_work_dir;
use anyhow::{anyhow, Context};
use owo_colors::OwoColorize;
use tokio::fs::rename;

async fn migrate_uvx_directory() -> anyhow::Result<()> {
    eprintln!(
        "Moving {} to {}.",
        "~/.local/uvx".yellow(),
        "~/.local/uvenv".green()
    );

    let workdir = get_work_dir();
    let old_workdir = workdir
        .parent()
        .ok_or_else(|| anyhow!("~/.local missing!"))?
        .join("uvx");

    rename(old_workdir, workdir).await?;

    Ok(())
}

impl Process for SelfMigrateOptions {
    async fn process(self) -> anyhow::Result<i32> {
        let mut error = Err(anyhow!("-> Possibly failed auto-migration."));
        let mut any_err = false;

        // move uvx directory:
        // note: this leads to invalid venvs, since the python symlinks are now incorrect.
        // also the ~/.local/bin symlinks will be broken.
        // `reinstall_all` fixes both of those issues.

        match migrate_uvx_directory().await {
            Ok(()) => {},
            Err(err) => {
                error = error
                    .with_context(|| "(while moving directory)")
                    .with_context(|| err);
                any_err = true;
            },
        }

        // reinstall to setup proper symlinks etc:

        match reinstall_all(None, true, false, false, false, &[]).await {
            Ok(()) => {},
            Err(err) => {
                error = error.with_context(|| err);
                any_err = true;
            },
        }

        if any_err {
            error.with_context(|| "Migration possibly failed;")
        } else {
            Ok(0)
        }
    }
}
