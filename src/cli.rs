use clap::{Parser, Subcommand};
use clap_complete::Shell;
use core::fmt::{Display, Formatter};

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
    async fn process(self) -> anyhow::Result<i32>;
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Parser)]
#[clap(version, styles=get_styles())]
pub struct Args {
    #[arg(long = "generate", value_enum)]
    pub generator: Option<Shell>,

    #[clap(subcommand)]
    pub cmd: Commands,
}

// impl Args {
//     pub fn parse_from_python() -> Args {
//         return Args::parse_from(env::args().skip(1)); // first argument is now 'python' instead of 'uvenv' so skip it
//     }
// }

const PYTHON_HELP_TEXT: &str =
    "Python version or executable to use, e.g. `3.12`, `python3.12`, `/usr/bin/python3.12`";

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Default, Parser)]
pub struct SetupOptions {
    #[clap(long, help = "Don't update $PATH in .bashrc/.zshrc")]
    pub skip_ensurepath: bool,
    #[clap(long, help = "Don't enable completions via .bashrc/.zshrc")]
    pub skip_completions: bool,
    #[clap(long, help = "Don't enable `uvenv activate` via .bashrc/.zshrc")]
    pub skip_activate: bool,
    #[clap(
        short,
        long,
        help = "Setup features without checking previous installation."
    )]
    pub force: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Default, Parser)]
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
    #[clap(
        long,
        help = "List only packages installed wwith a specific version of Python"
    )]
    pub python: Option<String>,
    pub venv_names: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Default, Parser)]
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
    #[clap(long, short, help = "Include extra dependencies")]
    pub with: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Default, Parser)]
pub struct CreateOptions {
    pub venv_name: String,
    #[clap(long, help = PYTHON_HELP_TEXT)]
    pub python: Option<String>,
    #[clap(long, help = "Skip installing basic packages like 'pip'")]
    pub no_seed: bool,
    #[clap(short, long, help = "Overwrite existing venv with conflicting name")]
    pub force: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Default, Parser)]
pub struct ActivateOptions {
    pub venv_name: String,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Default, Parser)]
pub struct UpgradeOptions {
    pub package_names: Vec<String>,
    #[clap(short = 'f', long, help = "Ignore previous version constraint")]
    pub force: bool,
    #[clap(long, help = "Don't also upgrade injected packages")]
    pub skip_injected: bool,
    #[clap(long, help = "Run without `uv` cache")]
    pub no_cache: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Default, Parser)]
pub struct UninstallOptions {
    pub package_name: String,
    #[clap(
        short = 'f',
        long,
        help = "Remove executable with the same name (in ~/.local/bin) even if related venv was not found."
    )]
    pub force: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Default, Parser)]
pub struct UninstallAllOptions {
    #[clap(
        short = 'f',
        long,
        help = "Remove executable with the same name (in ~/.local/bin) even if related venv was not found."
    )]
    pub force: bool,

    pub venv_names: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Default, Parser)]
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

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Default, Parser)]
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

    pub venv_names: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Default, Parser)]
pub struct UpgradeAllOptions {
    #[clap(short = 'f', long, help = "Ignore previous version constraint")]
    pub force: bool,
    #[clap(long, help = "Don't also upgrade injected packages")]
    pub skip_injected: bool,
    #[clap(long, help = "Run without `uv` cache")]
    pub no_cache: bool,

    pub venv_names: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Default, Parser)]
pub struct RunOptions {
    pub package_name: String,
    #[clap(long, help = "Run without `uv` cache")]
    pub no_cache: bool,
    #[clap(long, help = PYTHON_HELP_TEXT)]
    pub python: Option<String>,
    #[clap(long, help = "Don't remove the temporary venv when done running")]
    pub keep: bool,
    #[clap(long, short, help = "Include extra dependencies")]
    pub with: Vec<String>,
    #[clap(
        long,
        help = "Custom name of an executable to run (e.g. 'semantic-release' in the package 'python-semantic-release')"
    )]
    pub binary: Option<String>,
    pub args: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Default, Parser)]
pub struct RunuvOptions {
    pub venv: String,
    pub uv_args: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Default, Parser)]
pub struct RunpipOptions {
    pub venv: String,
    pub pip_args: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Default, Parser)]
pub struct RunpythonOptions {
    pub venv: String,
    pub python_args: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Default, clap::ValueEnum)]
pub enum OutputFormat {
    #[default]
    #[expect(clippy::upper_case_acronyms, reason = "Json just looks weird")]
    JSON,
    #[expect(clippy::upper_case_acronyms, reason = "Toml just looks weird")]
    TOML,
    Binary,
}

// impl OutputFormat {
//     pub fn to_string(&self) -> String {
//         match self {
//             Self::JSON => "json".to_owned(),
//             Self::TOML => "toml".to_owned(),
//             Self::Binary => "binary".to_owned(),
//         }
//     }
//     pub fn from_str(string: &str) -> Self {
//         match string.to_lowercase().as_ref() {
//             "toml" => Self::TOML,
//             "json" => Self::JSON,
//             "binary" | "msgpack" => Self::Binary,
//             other => {
//                 eprintln!("Unexpected format {other}! Using `toml` instead.");
//                 Self::TOML
//             },
//         }
//     }
// }

// includes to_string:
impl Display for OutputFormat {
    fn fmt(
        &self,
        f: &mut Formatter<'_>,
    ) -> core::fmt::Result {
        f.write_str(match self {
            Self::JSON => "json",
            Self::TOML => "toml",
            Self::Binary => "binary",
        })
    }
}

impl From<String> for OutputFormat {
    fn from(string: String) -> Self {
        // Self::from_str(&value)
        match string.to_lowercase().as_ref() {
            "toml" => Self::TOML,
            "json" => Self::JSON,
            "binary" | "msgpack" => Self::Binary,
            other => {
                eprintln!("Unexpected format {other}! Using `toml` instead.");
                Self::TOML
            },
        }
    }
}
impl From<OutputFormat> for String {
    fn from(value: OutputFormat) -> Self {
        value.to_string()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Default, Parser)]
pub struct FreezeOptions {
    /// The filename of the lockfile to generate. Defaults to `uvenv.lock`.
    #[clap(
        long,
        default_value = "uvenv.lock",
        help = "The filename of the lockfile to generate"
    )]
    pub filename: String,
    /// The version of the dependencies to freeze. Defaults to `latest`.
    #[clap(
        long,
        default_value = "latest",
        help = "The version of the lockfile format [options: latest (1)]",
        value_parser = validate_version
    )]
    pub version: Option<String>,
    /// A list of dependencies to include in the lockfile. Conflicts with `exclude`.
    #[clap(
        long,
        conflicts_with = "exclude",
        help = "A list of dependencies to include in the lockfile"
    )]
    pub include: Vec<String>,
    /// A list of dependencies to exclude from the lockfile. Conflicts with `include`.
    #[clap(
        long,
        conflicts_with = "include",
        help = "A list of dependencies to exclude from the lockfile"
    )]
    pub exclude: Vec<String>,
    /// The output format of the frozen dependencies. Defaults to `toml`.
    #[clap(
        long,
        default_value = "toml",
        help = "The output format of the frozen dependencies"
    )]
    pub format: OutputFormat,
}

fn validate_version(value: &str) -> Result<String, String> {
    match value {
        "1" | "latest" => Ok("1".to_owned()), // also convert 'latest' into '1'

        #[cfg(debug_assertions)]
        "0" => Ok("0".to_owned()),

        _ => Err("Invalid version. Only '1' and 'latest' are supported.".to_owned()),
    }
}

/// Options for the thaw command.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Default, Parser)]
pub struct ThawOptions {
    /// The filename of the lockfile to use..
    #[clap(
        long,
        default_value = "",
        help = "The filename of the lockfile to use (default: search in uvenv.lock, uvenv.toml, uvenv.json)"
    )]
    pub filename: String,
    /// A list of dependencies to include when thawing. Conflicts with `exclude`.
    #[clap(
        long,
        conflicts_with = "exclude",
        help = "A list of dependencies to include when thawing"
    )]
    pub include: Vec<String>,
    /// A list of dependencies to exclude when thawing. Conflicts with `include`.
    #[clap(
        long,
        conflicts_with = "include",
        help = "A list of dependencies to exclude when thawing"
    )]
    pub exclude: Vec<String>,
    /// Whether to remove the whole current environment before thawing. Defaults to `false`.
    #[clap(
        long,
        default_value = "false",
        help = "Remove the current environment before thawing",
        conflicts_with = "skip_current"
    )]
    pub remove_current: bool,
    /// Whether to overwrite existing dependencies when thawing. Defaults to `true`.
    #[clap(
        long,
        default_value = "false",
        help = "Don't overwrite existing dependencies when thawing",
        conflicts_with = "remove_current"
    )]
    pub skip_current: bool,

    #[clap(
        long,
        default_value = "frozen",
        help = "Which version of Python to use when thawing. [Options: frozen (default; use versions defined in lockfile), ignore (use default Python), <version> (specific Python version, e.g. 3.12, python3.12)]"
    )]
    pub python: String,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Default, Parser)]
pub struct EnsurepathOptions {
    #[clap(long, short, help = "Force update")]
    pub force: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Default, Parser)]
pub struct InjectOptions {
    pub into: String,
    pub package_specs: Vec<String>,

    #[clap(long, help = "Run without `uv` cache")]
    pub no_cache: bool,
}
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Default, Parser)]
pub struct UnInjectOptions {
    pub outof: String,
    pub package_specs: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Default, Parser)]
pub struct CheckOptions {
    #[clap(long, help = "Don't check if scripts are installed correctly.")]
    pub skip_scripts: bool,
    #[clap(long, help = "Don't check for updates", conflicts_with_all = ["show_prereleases", "ignore_constraints"])]
    pub skip_updates: bool,
    #[clap(long, help = "Don't check if scripts still have valid interpreter.")]
    pub skip_broken: bool,
    #[clap(long, help = "Show prerelease updates", conflicts_with_all = ["skip_updates"])]
    pub show_prereleases: bool,
    #[clap(long, help="Ignore version constraints when checking updates", conflicts_with_all = ["skip_updates"])]
    pub ignore_constraints: bool,

    #[clap(long, short, help = "Output as JSON")]
    pub json: bool,

    pub venv_names: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Default, Parser)]
pub struct CompletionsOptions {
    #[clap(long, short, help = "Add to ~/.bashrc (or ~/.zshrc)")]
    pub install: bool,
    // todo: support others than bash
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Parser)]
pub enum Commands {
    #[clap(about = "Setup additional (bash-specific) functionality.")]
    Setup(SetupOptions),
    #[clap(aliases=["ls"], about = "List packages and apps installed with uvenv.")]
    List(ListOptions),
    #[clap(aliases=["i"], about = "Install a package (by pip name).")]
    Install(InstallOptions),
    #[clap(aliases=["c"], about = "Create a new (empty) virtualenv")]
    Create(CreateOptions),
    #[clap(about = "Activate a uvenv-managed virtualenv (bash only)")]
    Activate(ActivateOptions),
    #[clap(aliases=["u", "update"], about = "Upgrade a package.")]
    Upgrade(UpgradeOptions),
    #[clap(about = "Upgrade all uvenv-installed packages.")]
    UpgradeAll(UpgradeAllOptions),
    #[clap(aliases = &["delete", "remove", "rm"], about = "Uninstall a package (by pip name).")]
    Uninstall(UninstallOptions),
    #[clap(aliases = &["remove-all", "delete-all"], about = "Uninstall all uvenv-installed packages.")]
    UninstallAll(UninstallAllOptions),
    #[clap(
        about = "Uninstall a package (by pip name) and re-install from the original spec (unless a new spec is supplied)."
    )]
    Reinstall(ReinstallOptions),
    #[clap(about = "Reinstall all uvenv-installed packages.")]
    ReinstallAll(ReinstallAllOptions),
    #[clap(aliases=["ij"], about = "Install additional packages to a virtual environment managed by uvenv.")]
    Inject(InjectOptions),
    #[clap(aliases = &["eject", "ej"], about="Uninstall additional packages from a virtual environment managed by uvenv. (alias: `eject`)")]
    Uninject(UnInjectOptions),
    #[clap(about = "Check for possible issues and updates.")]
    Check(CheckOptions),

    #[clap(aliases=["x"], about = "Run a package in a temporary virtual environment.")]
    Run(RunOptions),
    #[clap(about = "Run 'uv' in the right venv.")]
    Runuv(RunuvOptions),
    #[clap(about = "Run 'pip' in the right venv.")]
    Runpip(RunpipOptions),
    #[clap(about = "Run 'python' in the right venv.")]
    Runpython(RunpythonOptions),

    #[clap(about = "Create a lock file of all installed apps, which can be reinstalled via `thaw`")]
    Freeze(FreezeOptions),

    #[clap(about = "Install applications from a lockfile (usually `uvenv.lock`)")]
    Thaw(ThawOptions),

    #[clap(
        about = "Update ~/.bashrc (or ~/.zshrc) with a PATH that includes the local bin directory that uvenv uses."
    )]
    Ensurepath(EnsurepathOptions),
    #[clap(about = "Use --install to install the autocomplete script (bash).")]
    Completions(CompletionsOptions),
    #[clap(subcommand, about = "Manage uvenv itself.")]
    Self_(SelfCommands),
}

impl Process for Commands {
    async fn process(self) -> anyhow::Result<i32> {
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
            Self::Freeze(opts) => opts.process().await,
            Self::Thaw(opts) => opts.process().await,
            Self::Create(opts) => opts.process().await,
            Self::Self_(opts) => opts.process().await,
            Self::Check(opts) => opts.process().await,
        }
    }
}
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Default, Parser)]
pub struct SelfLinkOptions {
    #[clap(long, help = "Overwrite current symlink?")]
    pub force: bool,
    #[clap(long, help = "Don't produce output")]
    pub quiet: bool,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Default, Parser)]
pub struct SelfUpdateOptions {
    #[clap(long, help = "Update without also updating uv")]
    pub without_uv: bool,
    #[clap(long, help = "Update without also updating patchelf")]
    pub without_patchelf: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Default, Parser)]
pub struct SelfChangelogOptions;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Default, Parser)]
pub struct SelfMigrateOptions;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Default, Parser)]
pub struct SelfInfoOptions {
    #[clap(long, default_value = "true", help = "Show info about uvenv in a colored format (default)", conflicts_with_all = ["simple", "json", "toml"])]
    pub fancy: bool,

    #[clap(short, long, help = "Show info about uvenv in a basic format", conflicts_with_all = ["fancy", "json", "toml"])]
    pub simple: bool,

    #[clap(short, long, help = "Show info about uvenv in JSON format", conflicts_with_all = ["fancy", "simple", "toml"])]
    pub json: bool,

    #[clap(short, long, help = "Show info about uvenv in TOML format", conflicts_with_all = ["fancy", "simple", "json"])]
    pub toml: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Default, Parser)]
pub struct SelfVersionOptions;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Subcommand)]
pub enum SelfCommands {
    #[clap(about = "Update the current installation of uvenv (and optionally uv).")]
    Update(SelfUpdateOptions),

    #[clap(about = "Create a symlink for the current location of `uvenv` to ~/.local/bin/uvenv.")]
    Link(SelfLinkOptions),

    #[clap(about = "Show the uvenv changelog")]
    Changelog(SelfChangelogOptions),

    #[clap(about = "Migrate installed environments and commands from `uvx` to `uvenv`")]
    Migrate(SelfMigrateOptions),

    #[clap(about = "Show info about uvenv and it's dependencies")]
    Info(SelfInfoOptions),

    #[clap(about = "(deprecated in favor of `self info`)")]
    Version(SelfVersionOptions),
}

impl Process for SelfCommands {
    async fn process(self) -> anyhow::Result<i32> {
        match self {
            Self::Update(opts) => opts.process().await,
            Self::Link(opts) => opts.process().await,
            Self::Changelog(opts) => opts.process().await,
            Self::Migrate(opts) => opts.process().await,
            Self::Info(opts) => opts.process().await,
            Self::Version(opts) => opts.process().await,
        }
    }
}
