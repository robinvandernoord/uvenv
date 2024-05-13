mod animate;
mod cli;
mod cmd;
mod commands;
mod helpers;
mod metadata;
mod pip;
mod symlinks;
mod uv;
mod venv;

use std::io;

use clap::{Command, CommandFactory, Parser};
use clap_complete::{generate, Generator, Shell};

use crate::cli::{Args, Process};
use crate::commands::activate::install_activate;
use crate::commands::ensurepath::ensure_path_generate;

pub async fn print_completions<G: Generator>(
    gen: G,
    cmd: &mut Command,
) {
    generate(gen, cmd, cmd.get_name().to_string(), &mut io::stdout());
}

pub async fn generate_bash(generator: Shell) {
    let mut cmd = Args::command();

    let args = &cmd.clone().get_matches();
    match args.subcommand_name() {
        Some("activate") => {
            // generate code for uvx activate
            install_activate().await
        },
        Some("ensurepath") => {
            // geneate code for uvx ensurepath
            ensure_path_generate().await
        },
        _ => {
            // other cases: show regular completions
            print_completions(generator, &mut cmd).await;
            // todo: dynamic completions for e.g. `uvx upgrade <venv>`
        },
    }
}

pub async fn generate_code(target: Shell) -> i32 {
    match target {
        Shell::Bash => {
            generate_bash(target).await;
            0
        },
        _ => {
            eprintln!("Error: only 'bash' is supported at this moment.");
            126
        },
    }
}

#[tokio::main]
async fn main() {
    let args = Args::parse();

    let exit_code = if let Some(generator) = args.generator {
        generate_code(generator).await
    } else {
        let result = args.cmd.process().await;
        result.unwrap_or_else(|msg| {
            eprintln!("Something went wrong | {}", msg);
            1
        })
    };

    // If bundled via an entrypoint, the first argument is 'python' so skip it:
    // let args = Args::parse_from_python();

    std::process::exit(exit_code);
}
