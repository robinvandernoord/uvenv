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
use clap_complete::{generate, Generator};

use crate::cli::{Args, Process};

fn print_completions<G: Generator>(
    gen: G,
    cmd: &mut Command,
) {
    generate(gen, cmd, cmd.get_name().to_string(), &mut io::stdout());
}

#[tokio::main]
async fn main() {
    let args = Args::parse();

    let code: i32;

    if let Some(generator) = args.generator {
        let mut cmd = Args::command();
        print_completions(generator, &mut cmd);

        // todo: dynamic completions for e.g. `uvx upgrade <venv>`

        code = 0
    } else {
        code = args.cmd.process().await.unwrap_or_else(|msg| {
            eprintln!("Something went wrong | {}", msg);
            1
        });
    }

    // If bundled via an entrypoint, the first argument is 'python' so skip it:
    // let args = Args::parse_from_python();

    std::process::exit(code);
}
