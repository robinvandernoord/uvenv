use crate::cli::{CompletionsOptions, Process};
use anyhow::Context;

use crate::cmd::run_if_bash_else_warn;
use owo_colors::OwoColorize;

use super::ensurepath::add_to_bashrc;

pub async fn completions(install: bool) -> anyhow::Result<i32> {
    let for_bash = r#"eval "$(uvenv --generate=bash completions)""#;

    if install {
        // you probably want `uvenv setup` but keep this for legacy.
        add_to_bashrc(for_bash, true).await?;
        Ok(0)
    } else {
        Ok(run_if_bash_else_warn(|_shell| {
            eprintln!(
                "Tip: place this line in your ~/.bashrc or run '{}' to do this automatically!",
                "uvenv setup".green()
            );
            println!("{for_bash}");
            Some(0)
        })
        .unwrap_or(1))
    }
}

impl Process for CompletionsOptions {
    async fn process(self) -> anyhow::Result<i32> {
        completions(self.install)
            .await
            .with_context(|| "Something went wrong trying to generate or install completions;")
    }
}
