mod animate;
mod cli;
mod cmd;
mod commands;
mod helpers;
mod metadata;
mod pip;
mod promises;
mod pypi;
mod symlinks;
#[cfg(test)]
mod tests;
mod uv;
mod venv;

mod lockfile;
mod macros;
mod shell;

use std::io;

use clap::{Command, CommandFactory, Parser};
use clap_complete::{Generator, Shell, generate};

use crate::cli::{Args, Process};
use crate::commands::activate::generate_activate;
use crate::commands::ensurepath::ensure_path_generate;
use crate::helpers::fmt_error;
use crate::shell::SupportedShell;
use std::process::exit;

pub fn print_completions<G: Generator>(
    generator: G,
    cmd: &mut Command,
) {
    // get_name returns a str, to_owned = to_string (but restriction::str_to_string)
    generate(generator, cmd, cmd.get_name().to_owned(), &mut io::stdout());
}

pub async fn generate_completions_shell(generator: Shell) {
    let mut cmd = Args::command();

    let args = cmd.clone().get_matches();
    match args.subcommand_name() {
        Some("activate") => {
            // generate code for uvenv activate
            println!("{}", generate_activate().await);
        },
        Some("ensurepath") => {
            // geneate code for uvenv ensurepath
            println!("{}", ensure_path_generate().await);
        },
        _ => {
            // other cases: show regular completions
            // note: this should support zsh but doesn't seem to actually work :(
            print_completions(generator, &mut cmd);
            // todo: dynamic completions for e.g. `uvenv upgrade <venv>`
        },
    }
}

pub async fn generate_code(target: Shell) -> i32 {
    match target {
        Shell::Bash | Shell::Zsh => {
            generate_completions_shell(target).await;
            0
        },
        Shell::Elvish | Shell::Fish | Shell::PowerShell => {
            eprintln!(
                "Error: only '{}' are supported at this moment.",
                SupportedShell::list_options_formatted()
            );
            126
        },
        #[expect(
            clippy::todo,
            reason = "This is used in a catch all that should be exhaustive"
        )]
        _ => {
            todo!("Unknown shell, not implemented yet!");
        },
    }
}

#[tokio::main]
async fn main() {
    let args = Args::parse();

    let exit_code = if let Some(generator) = args.generator {
        generate_code(generator).await
    } else {
        args.cmd.process().await.unwrap_or_else(|msg| {
            eprintln!("{}", fmt_error(&msg));
            1
        })
    };

    // If bundled via an entrypoint, the first argument is 'python' so skip it:
    // let args = Args::parse_from_python();

    exit(exit_code);
}
