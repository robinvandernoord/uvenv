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
mod tests;
mod uv;
mod venv;

use std::io;

use clap::{Command, CommandFactory, Parser};
use clap_complete::{generate, Generator, Shell};

use crate::cli::{Args, Process};
use crate::commands::activate::generate_activate;
use crate::commands::ensurepath::ensure_path_generate;
use crate::helpers::fmt_error;
use std::process::exit;

pub fn print_completions<G: Generator>(
    gen: G,
    cmd: &mut Command,
) {
    // get_name returns a str, to_owned = to_string (but restriction::str_to_string)
    generate(gen, cmd, cmd.get_name().to_owned(), &mut io::stdout());
}

pub async fn generate_bash(generator: Shell) {
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
            print_completions(generator, &mut cmd);
            // todo: dynamic completions for e.g. `uvenv upgrade <venv>`
        },
    }
}

pub async fn generate_code(target: Shell) -> i32 {
    if target == Shell::Bash {
        generate_bash(target).await;
        0
    } else {
        eprintln!("Error: only 'bash' is supported at this moment.");
        126
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
