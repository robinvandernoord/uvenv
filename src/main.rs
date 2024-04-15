mod animate;
mod cli;
mod commands;
mod helpers;
mod metadata;
mod pip;
mod symlinks;
mod uv;
mod venv;

use clap::Parser;

use crate::cli::{Args, Process};

#[tokio::main]
async fn main() {
    let args = Args::parse();
    // If bundled via an entrypoint, the first argument is 'python' so skip it:
    // let args = Args::parse_from_python();

    let code = match args.cmd.process().await {
        Ok(code) => code,
        Err(msg) => {
            eprintln!("Something went wrong | {}", msg);
            1
        },
    };

    std::process::exit(code);
}
