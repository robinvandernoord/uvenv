use std::io;

use crate::cli::{Process, CompletionsOptions};
use clap::Command;
use clap_complete::{generate, Generator, Shell};

fn print_completions<G: Generator>(gen: G, cmd: &mut Command) {
    generate(gen, cmd, cmd.get_name().to_string(), &mut io::stdout());
}


impl Process for CompletionsOptions {
    async fn process(self) -> Result<i32, String> {
        // print_completions();
        return Ok(2);
    }
}
