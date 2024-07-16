use crate::cli::{ActivateOptions, Process};
use crate::cmd::run_if_bash_else_warn;
use crate::commands::ensurepath::add_to_bashrc;
use owo_colors::OwoColorize;

pub async fn generate_activate() -> &'static str {
    // Used by `uvenv --generate bash activate _`
    // note: only bash is supported right now!
    include_str!("../shell/activate.sh")
}

pub async fn install_activate() -> anyhow::Result<()> {
    let bash_code = r#"eval "$(uvenv --generate=bash activate _)""#;
    // call eval instead of actually adding the bash function() to bashrc
    // so updates are available immediately
    add_to_bashrc(bash_code, true).await
}

impl Process for ActivateOptions {
    async fn process(self) -> anyhow::Result<i32> {
        // wait a minute, this is not a bash script!
        // show warning with setup info:

        Ok(
            run_if_bash_else_warn(|shell| {
                println!("Your shell ({}) is supported, but the shell extension is not set up.\n\
                You can use `uvenv setup` to do this automatically, or add `{}` to your bashrc file to enable it manually.",
                         &shell.blue(),
                         r#"eval "$(uvenv --generate=bash activate _)""#.green()
                );
                Some(1)
            }).unwrap_or(126) // = cannot execute, if not BASH
        )
    }
}
