use crate::cli::{Process, SetupOptions};
use crate::cmd::run_if_bash_else_warn;
use crate::commands::activate::install_activate;
use crate::commands::completions::completions;
use crate::commands::ensurepath::ensure_path;
use owo_colors::OwoColorize;

pub async fn setup_for_bash(
    do_ensurepath: bool,
    do_completions: bool,
    do_activate: bool,
) -> Result<i32, String> {
    let mut any_warnings: bool = false;

    if do_ensurepath {
        if let Err(msg) = ensure_path(false).await {
            any_warnings = true;
            eprintln!("{}", msg);
        }
    }

    if do_completions {
        if let Err(msg) = completions(true).await {
            any_warnings = true;
            eprintln!("{}", msg);
        }
    }

    if do_activate {
        if let Err(msg) = install_activate().await {
            any_warnings = true;
            eprintln!("{}", msg);
        }
    }

    Ok(if any_warnings {
        1
    } else {
        println!("{}", "Setup complete!".green());
        0
    })
}

impl Process for SetupOptions {
    async fn process(self) -> Result<i32, String> {
        let result = run_if_bash_else_warn(move |_| {
            // some logic here
            let result = setup_for_bash(
                !self.skip_ensurepath,
                !self.skip_completions,
                !self.skip_activate,
            );

            // async is not possible in this block,
            // creating a run_if_bash_else_warn_async is non-trivial
            Some(result) // so just return a promise
        });

        match result {
            Some(promise) => {
                // finally, we can await
                promise.await
            },
            None => {
                // unsupported shell ->
                Ok(126)
            },
        }
    }
}
