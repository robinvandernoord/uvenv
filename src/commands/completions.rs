use crate::cli::{CompletionsOptions, Process};

use crate::cmd::run_if_bash_else_warn;
use owo_colors::OwoColorize;

use super::ensurepath::add_to_bashrc;

pub async fn completions(install: bool) -> Result<i32, String> {
    let for_bash = r#"eval "$(uvx --generate=bash completions)""#;

    if install {
        // you probably want `uvx setup` but keep this for legacy.
        add_to_bashrc(for_bash, true).await?;
        Ok(0)
    } else {
        Ok(run_if_bash_else_warn(|_shell| {
            eprintln!(
                "Tip: place this line in your ~/.bashrc or run '{}' to do this automatically!",
                "uvx setup".green()
            );
            println!("{for_bash}");
            Some(0)
        })
        .unwrap_or(1))
    }
}

impl Process for CompletionsOptions {
    async fn process(self) -> Result<i32, String> {
        completions(self.install).await
    }
}
