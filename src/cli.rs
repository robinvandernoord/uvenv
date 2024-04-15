use clap::{Parser, Subcommand};

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

pub trait Process {
    async fn process(self) -> Result<i32, String>;
}

#[derive(Parser, Debug)]
#[clap(version, styles=get_styles())]
pub struct Args {
    #[clap(subcommand)]
    pub cmd: Commands,
}

// impl Args {
//     pub fn parse_from_python() -> Args {
//         return Args::parse_from(env::args().skip(1)); // first argument is now 'python' instead of 'uvx' so skip it
//     }
// }

const PYTHON_HELP_TEXT: &str =
    "Python version or executable to use, e.g. `3.12`, `python3.12`, `/usr/bin/python3.12`";

#[derive(Debug, Parser)]
pub struct ListOptions {
    #[clap(short, long, help = "Short output", conflicts_with_all = ["verbose"])]
    pub short: bool,
    #[clap(short, long, help = "Verbose output", conflicts_with_all = ["short", "json"])]
    pub verbose: bool,
    #[clap(short, long, help = "Output in JSON format", conflicts_with_all = ["verbose"])]
    pub json: bool,

    pub venv_names: Vec<String>,
}

#[derive(Debug, Parser)]
pub struct InstallOptions {
    pub package_name: String,
    #[clap(
        short = 'f',
        long,
        help = "Overwrite currently installed executables with the same name (in ~/.local/bin)"
    )]
    pub force: bool,
    #[clap(long, help = "Run without `uv` cache")]
    pub no_cache: bool,
    #[clap(long, help = PYTHON_HELP_TEXT)]
    pub python: Option<String>,
    #[clap(long, short, help = "Editable Install")]
    pub editable: bool,
}

#[derive(Debug, Parser)]
pub struct UpgradeOptions {
    pub package_name: String,
    #[clap(short = 'f', long, help = "Ignore previous version constraint")]
    pub force: bool,
    #[clap(long, help = "Don't also upgrade injected packages")]
    pub skip_injected: bool,
    #[clap(long, help = "Run without `uv` cache")]
    pub no_cache: bool,
}

#[derive(Debug, Parser)]
pub struct UninstallOptions {
    pub package_name: String,
    #[clap(
        short = 'f',
        long,
        help = "Remove executable with the same name (in ~/.local/bin) even if related venv was not found"
    )]
    pub force: bool,
}

#[derive(Debug, Parser)]
pub struct ReinstallOptions {
    pub package: String,
    #[clap(long, help = PYTHON_HELP_TEXT)]
    pub python: Option<String>,
    #[clap(short = 'f', long, help = "See `install --force`")]
    pub force: bool,
    #[clap(
        long,
        help = "Don't include previously injected libraries in reinstall"
    )]
    pub without_injected: bool,
    #[clap(long, help = "Run without `uv` cache")]
    pub no_cache: bool,
    #[clap(long, short, help = "(Re)install as editable")]
    pub editable: bool,
}

#[derive(Debug, Parser)]
pub struct UpgradeAllOptions {
    #[clap(short = 'f', long, help = "Ignore previous version constraint")]
    force: bool,
    #[clap(long, help = "Don't also upgrade injected packages")]
    skip_injected: bool,
    #[clap(long, help = "Run without `uv` cache")]
    no_cache: bool,
}

#[derive(Debug, Parser)]
pub struct RunuvOptions {
    pub venv: String,
    pub uv_args: Vec<String>,
}

#[derive(Debug, Parser)]
pub struct RunpipOptions {
    pub venv: String,
    pub pip_args: Vec<String>,
}

#[derive(Debug, Parser)]
pub struct RunpythonOptions {
    pub venv: String,
    pub python_args: Vec<String>,
}

#[derive(Debug, Parser)]
pub struct EnsurepathOptions {
    #[clap(long, short, help = "Force update")]
    pub force: bool,
}

#[derive(Debug, Parser)]
pub struct SelfUpdateOptions {
    #[clap(long, help = "Update with uv")]
    with_uv: Option<bool>,
}

#[derive(Debug, Parser)]
pub struct InjectOptions {
    into: String,
    package_specs: Vec<String>,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    List(ListOptions),
    Install(InstallOptions),
    Upgrade(UpgradeOptions),
    Uninstall(UninstallOptions),
    Reinstall(ReinstallOptions),
    Inject(InjectOptions),
    UpgradeAll(UpgradeAllOptions),
    Runuv(RunuvOptions),
    Runpip(RunpipOptions),
    Runpython(RunpythonOptions),
    Ensurepath(EnsurepathOptions),
    SelfUpdate(SelfUpdateOptions),
}

impl Process for Commands {
    async fn process(self) -> Result<i32, String> {
        return match self {
            Commands::List(opts) => opts.process().await,
            Commands::Install(opts) => opts.process().await,
            Commands::Upgrade(opts) => opts.process().await,
            Commands::Uninstall(opts) => opts.process().await,
            Commands::Reinstall(opts) => opts.process().await,
            Commands::Inject(opts) => opts.process().await,
            Commands::UpgradeAll(opts) => opts.process().await,
            Commands::Runuv(opts) => opts.process().await,
            Commands::Runpip(opts) => opts.process().await,
            Commands::Runpython(opts) => opts.process().await,
            Commands::Ensurepath(opts) => opts.process().await,
            Commands::SelfUpdate(opts) => opts.process().await,
        };
    }
}
