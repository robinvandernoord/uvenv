use std::env;

use clap::{Parser, Subcommand};
use pyo3::prelude as pyo;


pub fn get_styles() -> clap::builder::Styles {
    clap::builder::Styles::styled()
        .usage(
            anstyle::Style::new()
                .bold()
                .underline()
                .fg_color(Some(anstyle::Color::Ansi(anstyle::AnsiColor::Yellow))),
        )
        .header(
            anstyle::Style::new()
                .bold()
                .underline()
                .fg_color(Some(anstyle::Color::Ansi(anstyle::AnsiColor::Yellow))),
        )
        .literal(
            anstyle::Style::new().fg_color(Some(anstyle::Color::Ansi(anstyle::AnsiColor::Green))),
        )
        .invalid(
            anstyle::Style::new()
                .bold()
                .fg_color(Some(anstyle::Color::Ansi(anstyle::AnsiColor::Red))),
        )
        .error(
            anstyle::Style::new()
                .bold()
                .fg_color(Some(anstyle::Color::Ansi(anstyle::AnsiColor::Red))),
        )
        .valid(
            anstyle::Style::new()
                .bold()
                .underline()
                .fg_color(Some(anstyle::Color::Ansi(anstyle::AnsiColor::Green))),
        )
        .placeholder(
            anstyle::Style::new().fg_color(Some(anstyle::Color::Ansi(anstyle::AnsiColor::White))),
        )
}

#[derive(Parser, Debug)]
#[clap(version, styles=get_styles())]
struct Args {
    #[clap(subcommand)]
    cmd: Commands,
}

const PYTHON_HELP_TEXT: &str = "Python version or executable to use, e.g. `3.12`, `python3.12`, `/usr/bin/python3.12`";

#[derive(Subcommand, Debug)]
enum Commands {
    List {
        #[clap(short, long, help = "Short output")]
        short: bool,
        #[clap(long, help = "Verbose output")]
        verbose: bool,
        #[clap(long, help = "Output in JSON format")]
        json: bool,
    },
    Install {
        package_name: String,
        #[clap(short = 'f', long, help = "Overwrite currently installed executables with the same name (in ~/.local/bin)")]
        force: bool,
        #[clap(long, help = "Run without `uv` cache")]
        no_cache: bool,
        #[clap(long, help = PYTHON_HELP_TEXT)]
        python: Option<String>,
    },
    Upgrade {
        package_name: String,
        #[clap(short = 'f', long, help = "Ignore previous version constraint")]
        force: bool,
        #[clap(long, help = "Don't also upgrade injected packages")]
        skip_injected: bool,
        #[clap(long, help = "Run without `uv` cache")]
        no_cache: bool,
    },
    Uninstall {
        package_name: String,
        #[clap(short = 'f', long, help = "Remove executable with the same name (in ~/.local/bin) even if related venv was not found")]
        force: bool,
    },
    Reinstall {
        package: String,
        #[clap(long, help = PYTHON_HELP_TEXT)]
        python: Option<String>,
        #[clap(short = 'f', long, help = "See `install --force`")]
        force: bool,
        #[clap(long, help = "Don't include previously injected libraries in reinstall")]
        without_injected: bool,
        #[clap(long, help = "Run without `uv` cache")]
        no_cache: bool,
    },
    Inject {
        into: String,
        package_specs: Vec<String>,
    },
    UpgradeAll {
        #[clap(short = 'f', long, help = "Ignore previous version constraint")]
        force: bool,
        #[clap(long, help = "Don't also upgrade injected packages")]
        skip_injected: bool,
        #[clap(long, help = "Run without `uv` cache")]
        no_cache: bool,
    },
    Runuv {
        venv: String,
    },
    Runpip {
        venv: String,
    },
    Runpython {
        venv: String,
    },
    Ensurepath {
        #[clap(long, help = "Force update")]
        force: bool,
    },
    SelfUpdate {
        #[clap(long, help = "Update with uv")]
        with_uv: Option<bool>,
    },
}


async fn async_main_rs() -> u32 {
    let args = Args::parse_from(env::args().skip(1)); // first argument is now 'python' instead of 'uvx' so skip it
    dbg!(args);
    return 0
}


#[pyo::pyfunction]
fn main_rs(py: pyo::Python<'_>) -> pyo::PyResult<&pyo::PyAny> {
    pyo3_asyncio::tokio::future_into_py(py, async {
        let exit_code = async_main_rs().await;

        Ok(pyo::Python::with_gil(|_| exit_code))
    })
}

/// A Python module implemented in Rust.
#[pyo::pymodule]
fn uvx(_py: pyo::Python, m: &pyo::PyModule) -> pyo::PyResult<()> {
    m.add_function(pyo::wrap_pyfunction!(main_rs, m)?)?;
    Ok(())
}
