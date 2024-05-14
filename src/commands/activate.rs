use crate::cli::{ActivateOptions, Process};
use crate::cmd::run_if_bash_else_warn;
use owo_colors::OwoColorize;

pub async fn install_activate() {
    // Used by `uvx --generate bash activate _`
    // note: only bash is supported right now!
    println!("{}", include_str!("../shell/activate.sh"));
}

impl Process for ActivateOptions {
    async fn process(self) -> Result<i32, String> {
        // wait a minute, this is not a bash script!
        // show warning with setup info:

        Ok(
            run_if_bash_else_warn(|shell| {
                println!("Your shell ({}) is supported, but the shell extension is not set up.\n\
                You can use `uvx setup` to do this automatically, or add `{}` to your bashrc file to enable it manually.",
                         &shell.blue(),
                         r#"eval "$(uvx --generate=bash activate _)""#.green()
                );
                Some(1)
            }).unwrap_or(126) // = cannot execute, if not BASH
        )
    }
}
