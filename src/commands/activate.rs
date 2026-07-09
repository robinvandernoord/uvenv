use crate::cli::{ActivateOptions, Process};
use crate::shell::{SupportedShell, run_if_supported_shell_else_warn};
use owo_colors::OwoColorize;

pub async fn generate_activate() -> &'static str {
    // Used by `uvenv --generate bash/zsh activate _`
    let shell = SupportedShell::detect();
    shell.activation_script()
}

pub async fn install_activate() -> anyhow::Result<()> {
    let shell = SupportedShell::detect();
    let sh_code = format!(r#"eval "$(uvenv --generate={} activate _)""#, shell.name());
    // call eval instead of actually adding the shell function() to bashrc/zshrc
    // so updates are available immediately
    shell.add_to_rcfile(&sh_code, true).await
}

impl Process for ActivateOptions {
    async fn process(self) -> anyhow::Result<i32> {
        Ok(
            run_if_supported_shell_else_warn(|shell| {
                println!("Your shell ({}) is supported, but the shell extension is not set up.\n\
                You can use `uvenv setup` to do this automatically, or add `{}` to your shell's configuration file to enable it manually.",
                         shell.blue(),
                         format!(r#"eval "$(uvenv --generate={shell} activate _)""#).green()
                );
                Some(1)
            }).unwrap_or(126) // Return 126 if shell is unsupported
        )
    }
}
