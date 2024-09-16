use crate::cli::{Process, SelfLinkOptions};
use crate::helpers::PathAsStr;
use crate::metadata::ensure_bin_dir;
use anyhow::Context;
use owo_colors::OwoColorize;
use std::env;
use tokio::fs::symlink;

pub async fn self_link(
    force: bool,
    quiet: bool,
) -> anyhow::Result<i32> {
    let bin = ensure_bin_dir().await;
    let uvenv = bin.join("uvenv");
    let current = env::current_exe()?;

    if uvenv == current {
        // nothing to do
        Ok(0)
    } else if uvenv.exists() && !force {
        if !quiet {
            eprintln!(
                "{}: {} already exists. Use '--force' to overwrite.",
                "Warning".yellow(),
                uvenv.as_str().green()
            );
        }
        // don't bail/Err because it's just a warning.
        // still exit with code > 0
        Ok(2) // missing -f
    } else {
        symlink(&current, &uvenv)
            .await
            .with_context(|| format!("Failed to create symlink {:?} -> {:?}", &uvenv, &current))?;

        Ok(0)
    }
}

impl Process for SelfLinkOptions {
    async fn process(self) -> anyhow::Result<i32> {
        let result = self_link(self.force, self.quiet).await;

        if self.quiet {
            // don't complain
            Ok(0)
        } else {
            result.with_context(|| "Something went wrong trying to symlink 'uvenv';")
        }
    }
}
