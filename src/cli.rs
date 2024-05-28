use clap::{Parser, Subcommand};
use clap_complete::Shell;

pub const fn get_styles() -> clap::builder::Styles {
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
    #[arg(long = "generate", value_enum)]
    pub generator: Option<Shell>,

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
pub struct SetupOptions {
    #[clap(long, help = "Don't update $PATH in .bashrc")]
    pub skip_ensurepath: bool,
    #[clap(long, help = "Don't enable completions via .bashrc")]
    pub skip_completions: bool,
    #[clap(long, help = "Don't enable `uvx activate` via .bashrc")]
    pub skip_activate: bool,
    #[clap(
        short,
        long,
        help = "Setup features without checking previous installation."
    )]
    pub force: bool,
}

#[derive(Debug, Parser)]
pub struct ListOptions {
    #[clap(short, long, help = "Short output", conflicts_with_all = ["verbose"])]
    pub short: bool,
    #[clap(short, long, help = "Verbose output", conflicts_with_all = ["short", "json"])]
    pub verbose: bool,
    #[clap(short, long, help = "Output in JSON format", conflicts_with_all = ["verbose"])]
    pub json: bool,

    #[clap(long, help = "Don't check for updates", conflicts_with_all = ["show_prereleases", "ignore_constraints"])]
    pub skip_updates: bool,
    #[clap(long, help = "Show prerelease updates", conflicts_with_all = ["skip_updates"])]
    pub show_prereleases: bool,
    #[clap(long, help="Ignore version constraints when checking updates", conflicts_with_all = ["skip_updates"])]
    pub ignore_constraints: bool,

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
pub struct CreateOptions {
    pub venv_name: String,
    #[clap(long, help = PYTHON_HELP_TEXT)]
    pub python: Option<String>,
    #[clap(long, help = "Skip installing basic packages like 'pip'")]
    pub no_seed: bool,
    #[clap(short, long, help = "Overwrite existing venv with conflicting name")]
    pub force: bool,
}

#[derive(Debug, Parser)]
pub struct ActivateOptions {
    pub venv_name: String,
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
        help = "Remove executable with the same name (in ~/.local/bin) even if related venv was not found."
    )]
    pub force: bool,
}

#[derive(Debug, Parser)]
pub struct UninstallAllOptions {
    #[clap(
        short = 'f',
        long,
        help = "Remove executable with the same name (in ~/.local/bin) even if related venv was not found."
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
pub struct ReinstallAllOptions {
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
    pub force: bool,
    #[clap(long, help = "Don't also upgrade injected packages")]
    pub skip_injected: bool,
    #[clap(long, help = "Run without `uv` cache")]
    pub no_cache: bool,
}

#[derive(Debug, Parser)]
pub struct RunOptions {
    pub package_name: String,
    #[clap(long, help = "Run without `uv` cache")]
    pub no_cache: bool,
    #[clap(long, help = PYTHON_HELP_TEXT)]
    pub python: Option<String>,
    #[clap(long, help = "Don't remove the temporary venv when done running")]
    pub keep: bool,
    #[clap(
        long,
        help = "Custom name of an executable to run (e.g. 'semantic-release' in the package 'python-semantic-release')"
    )]
    pub binary: Option<String>,
    pub args: Vec<String>,
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
pub struct InjectOptions {
    pub into: String,
    pub package_specs: Vec<String>,

    #[clap(long, help = "Run without `uv` cache")]
    pub no_cache: bool,
}
#[derive(Debug, Parser)]
pub struct UnInjectOptions {
    pub outof: String,
    pub package_specs: Vec<String>,
}

#[derive(Debug, Parser)]
pub struct CheckOptions {
    #[clap(long, help = "Don't check if scripts are installed correctly.")]
    pub skip_scripts: bool,
    #[clap(long, help = "Don't check for updates", conflicts_with_all = ["show_prereleases", "ignore_constraints"])]
    pub skip_updates: bool,
    #[clap(long, help = "Show prerelease updates", conflicts_with_all = ["skip_updates"])]
    pub show_prereleases: bool,
    #[clap(long, help="Ignore version constraints when checking updates", conflicts_with_all = ["skip_updates"])]
    pub ignore_constraints: bool,

    #[clap(long, short, help = "Output as JSON")]
    pub json: bool,

    pub venv_names: Vec<String>,
}

#[derive(Debug, Parser)]
pub struct CompletionsOptions {
    #[clap(long, short, help = "Add to ~/.bashrc")]
    pub install: bool,
    // todo: support others than bash
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    #[clap(about = "Setup additional (bash-specific) functionality.")]
    Setup(SetupOptions),
    #[clap(about = "List packages and apps installed with uvx.")]
    List(ListOptions),
    #[clap(about = "Install a package (by pip name).")]
    Install(InstallOptions),
    #[clap(about = "Create a new (empty) virtualenv")]
    Create(CreateOptions),
    #[clap(about = "Activate an uvx-managed virtualenv (bash only)")]
    Activate(ActivateOptions),
    #[clap(about = "Upgrade a package.")]
    Upgrade(UpgradeOptions),
    #[clap(about = "Upgrade all uvx-installed packages.")]
    UpgradeAll(UpgradeAllOptions),
    #[clap(aliases = &["delete", "remove"], about = "Uninstall a package (by pip name).")]
    Uninstall(UninstallOptions),
    #[clap(about = "Uninstall all uvx-installed packages.")]
    UninstallAll(UninstallAllOptions),
    #[clap(
        about = "Uninstall a package (by pip name) and re-install from the original spec (unless a new spec is supplied)."
    )]
    Reinstall(ReinstallOptions),
    #[clap(about = "Reinstall all uvx-installed packages.")]
    ReinstallAll(ReinstallAllOptions),
    #[clap(about = "Install additional packages to a virtual environment managed by uvx.")]
    Inject(InjectOptions),
    #[clap(aliases = &["eject"], about="Uninstall additional packages from a virtual environment managed by uvx. (alias: `eject`)")]
    Uninject(UnInjectOptions),
    #[clap(about = "Check for possible issues and updates.")]
    Check(CheckOptions),

    #[clap(about = "Run a package in a temporary virtual environment.")]
    Run(RunOptions),
    #[clap(about = "Run 'uv' in the right venv.")]
    Runuv(RunuvOptions),
    #[clap(about = "Run 'pip' in the right venv.")]
    Runpip(RunpipOptions),
    #[clap(about = "Run 'python' in the right venv.")]
    Runpython(RunpythonOptions),
    #[clap(
        about = "Update ~/.bashrc with a PATH that includes the local bin directory that uvx uses."
    )]
    Ensurepath(EnsurepathOptions),
    #[clap(about = "Use --install to install the autocomplete script (bash).")]
    Completions(CompletionsOptions),

    #[clap(subcommand, about = "Manage uvx itself.")]
    Self_(SelfCommands),
}

// todo `uvx check`:
//   - show missing metadata
//   - show packages with updates
//   - show packages with script issues

impl Process for Commands {
    async fn process(self) -> Result<i32, String> {
        match self {
            Self::List(opts) => opts.process().await,
            Self::Install(opts) => opts.process().await,
            Self::Upgrade(opts) => opts.process().await,
            Self::Uninstall(opts) => opts.process().await,
            Self::Reinstall(opts) => opts.process().await,
            Self::Inject(opts) => opts.process().await,
            Self::Activate(opts) => opts.process().await,
            Self::UpgradeAll(opts) => opts.process().await,
            Self::Runuv(opts) => opts.process().await,
            Self::Runpip(opts) => opts.process().await,
            Self::Runpython(opts) => opts.process().await,
            Self::Ensurepath(opts) => opts.process().await,
            Self::UninstallAll(opts) => opts.process().await,
            Self::ReinstallAll(opts) => opts.process().await,
            Self::Uninject(opts) => opts.process().await,
            Self::Completions(opts) => opts.process().await,
            Self::Run(opts) => opts.process().await,
            Self::Setup(opts) => opts.process().await,
            Self::Create(opts) => opts.process().await,
            Self::Self_(opts) => opts.process().await,
            Self::Check(opts) => opts.process().await,
        }
    }
}

#[derive(Debug, Parser)]
pub struct SelfUpdateOptions {
    #[clap(long, help = "Update without also updating uv")]
    pub without_uv: bool,
}

#[derive(Debug, Parser)]
pub struct ChangelogOptions {}

#[derive(Subcommand, Debug)]
pub enum SelfCommands {
    #[clap(about = "Update the current installation of uvx (and optionally uv).")]
    Update(SelfUpdateOptions),

    #[clap(about = "Show the uvx changelog")]
    Changelog(ChangelogOptions),
}

impl Process for SelfCommands {
    async fn process(self) -> Result<i32, String> {
        match self {
            Self::Update(opts) => opts.process().await,
            Self::Changelog(opts) => opts.process().await,
        }
    }
}
