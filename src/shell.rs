use crate::commands::ensurepath::{append, now};
use crate::helpers::Touch;
use crate::macros::iterable_enum_macro::iterable_enum;
use crate::metadata::get_home_dir;
use anyhow::{Context, anyhow, bail};
use core::fmt::{Display, Formatter, Write};
use owo_colors::OwoColorize;
use std::env;

iterable_enum! {
    #[derive(Debug)]
    pub enum SupportedShell {
        Bash,
        Zsh,
        Unsupported,
        }
}

impl SupportedShell {
    /// Detect the current shell based on the SHELL environment variable.
    pub fn detect() -> Self {
        let shell = env::var("SHELL").ok().unwrap_or_default();
        if shell.ends_with("bash") {
            Self::Bash
        } else if shell.ends_with("zsh") {
            Self::Zsh
        } else {
            Self::Unsupported
        }
    }

    /// Return the shell name.
    pub const fn name(&self) -> &'static str {
        match self {
            Self::Bash => "bash",
            Self::Zsh => "zsh",
            Self::Unsupported => "",
        }
    }

    pub const fn activation_script(&self) -> &'static str {
        match self {
            Self::Bash => {
                include_str!("./shells/bash.sh")
            },
            Self::Zsh => {
                include_str!("./shells/zsh.sh")
            },
            Self::Unsupported => "",
        }
    }

    /// Check if the shell is supported.
    pub const fn is_supported(&self) -> bool {
        !matches!(self, Self::Unsupported)
    }

    /// Get the appropriate rc file for each shell.
    pub const fn rc_file(&self) -> Option<&'static str> {
        match self {
            Self::Bash => Some(".bashrc"),
            Self::Zsh => Some(".zshrc"),
            Self::Unsupported => None,
        }
    }
    /// Add a modification to the appropriate rc file.
    pub async fn add_to_rcfile(
        &self,
        text: &str,
        with_comment: bool,
    ) -> anyhow::Result<()> {
        if let Some(rc_file) = self.rc_file() {
            add_to_rcfile(text, with_comment, rc_file).await
        } else {
            Err(anyhow!("Unsupported shell")).with_context(|| String::from(self.name()))
        }
    }

    /// Add a path modification to the appropriate rc file.
    pub async fn add_to_path(
        &self,
        path_to_add: &str,
        with_comment: bool,
    ) -> anyhow::Result<()> {
        let export_line = format!(r#"export PATH="$PATH:{path_to_add}""#);
        self.add_to_rcfile(&export_line, with_comment).await
    }

    pub fn list_options() -> Vec<&'static str> {
        Self::into_iter()
            .filter(Self::is_supported)
            .map(|it| it.name())
            .collect()
    }

    pub fn list_options_formatted() -> String {
        Self::list_options().join(" | ")
    }
}

impl Display for SupportedShell {
    fn fmt(
        &self,
        f: &mut Formatter<'_>,
    ) -> core::fmt::Result {
        write!(f, "{}", self.name())
    }
}

pub async fn add_to_rcfile(
    text: &str,
    with_comment: bool,
    filename: &str,
) -> anyhow::Result<()> {
    if cfg!(feature = "snap") {
        bail!(
            "{} snap-installed {} cannot write directly to `{}`. You can add the following line to make this feature work:\n\n{text}\n",
            "Warning:".yellow(),
            "`uvenv`".blue(),
            filename.blue()
        );
    }

    let path = get_home_dir().join(filename);
    path.touch()?; // ensure it exists

    let now = now();
    let mut final_text = String::from("\n");
    if with_comment {
        // final_text.push_str(&format!("# Added by `uvenv` at {now}\n"));
        writeln!(final_text, "# Added by `uvenv` at {now}")?;
    }

    final_text.push_str(text);
    final_text.push('\n');

    append(&path, &final_text)
        .await
        .with_context(|| format!("Trying to append text to your {filename}"))
}

/// Run a callback function if the shell is supported,
/// or show a message saying the shell is unsupported.
pub fn run_if_supported_shell_else_warn<T, Y: Fn(&SupportedShell) -> Option<T>>(
    if_supported: Y
) -> Option<T> {
    let shell = SupportedShell::detect();

    if shell.is_supported() {
        if_supported(&shell)
    } else {
        eprintln!(
            "Unsupported shell '{}'. Currently, these shells are supported: {}",
            shell.name().blue(),
            SupportedShell::list_options_formatted(),
        );
        None
    }
}

/// Run an async callback function if the shell is supported,
/// or show a message saying the shell is unsupported.
pub async fn run_if_supported_shell_else_warn_async<
    T,
    Y: AsyncFn(&'_ SupportedShell) -> Option<T>,
>(
    if_supported: Y
) -> Option<T> {
    let shell = SupportedShell::detect();

    if shell.is_supported() {
        if_supported(&shell).await
    } else {
        eprintln!(
            "Unsupported shell '{}'. Currently, these shells are supported: {}",
            shell.name().blue(),
            SupportedShell::list_options_formatted(),
        );
        None
    }
}
