use crate::cli::{CompletionsOptions, Process};

use owo_colors::OwoColorize;

use super::ensurepath::add_to_bashrc;

impl Process for CompletionsOptions {
    async fn process(self) -> Result<i32, String> {
        let for_bash = r#"eval "$(uvx --generate=bash completions)""#;

        if self.install {
            add_to_bashrc(for_bash, true).await?;
        } else {
            eprintln!(
                "Tip: place this line in your ~/.bashrc or run with '{}' to do this automatically!",
                "--install".green()
            );
            println!("{}", for_bash);
        }

        Ok(0)
    }
}
