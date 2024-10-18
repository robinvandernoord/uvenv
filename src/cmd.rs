use anyhow::anyhow;
use std::env;
use std::ffi::OsStr;
use std::path::PathBuf;
use std::process::{Output, Stdio};

use owo_colors::OwoColorize;
use tokio::fs::canonicalize;
use tokio::process::Command;

pub async fn find_sibling(name: &str) -> Option<PathBuf> {
    let Ok(binary_path) = &env::current_exe() else {
        return None;
    };

    let Ok(real_path) = canonicalize(&binary_path).await else {
        return None;
    };

    let parent = real_path.parent()?;
    // resolve symlinks etc:

    let binary = parent.join(name);

    // .then(|| binary) is the same:
    binary.exists().then_some(binary) // else None
}

pub async fn run_print_output<S1: AsRef<OsStr>, S2: AsRef<OsStr>>(
    command: S1,
    args: &[S2],
) -> anyhow::Result<i32> {
    let mut cmd = Command::new(command);
    cmd.args(args);
    cmd.stdout(Stdio::inherit());
    cmd.stderr(Stdio::inherit());
    let code = cmd.status().await?;

    Ok(code.code().unwrap_or(-1))
}

pub async fn run_get_output<S1: AsRef<OsStr>, S2: AsRef<OsStr>>(
    command: S1,
    args: &[S2],
) -> anyhow::Result<String> {
    let command_result = Command::new(command).args(args).output().await;

    match command_result {
        Ok(result) => match result.status.code() {
            Some(0) => Ok(String::from_utf8(result.stdout).unwrap_or_default()),
            Some(_) | None => Err(anyhow!(String::from_utf8(result.stderr).unwrap_or_default())),
        },
        Err(result_err) => Err(result_err.into()),
    }
}

pub async fn run<S1: AsRef<OsStr>, S2: AsRef<OsStr>>(
    script: S1,
    args: &[S2],
    err_prefix: Option<String>,
) -> anyhow::Result<bool> {
    let command_result = Command::new(script).args(args).output().await;

    #[expect(
        clippy::option_if_let_else,
        reason = "map_or_else complains about moved 'err'"
    )]
    match command_result {
        Ok(Output { status, stderr, .. }) => {
            if status.success() {
                Ok(true)
            } else {
                Err(String::from_utf8(stderr).unwrap_or_default())
            }
        },
        Err(result) => Err(result.to_string()),
    } // if err, add prefix:
    .map_err(|err| {
        if let Some(prefix) = err_prefix {
            anyhow!("{prefix} | {err}")
        } else {
            anyhow!(err)
        }
    })
}

/// Given a target shell (e.g. 'bash'), run a positive or negative callback function.
pub fn run_if_shell<T, Y: Fn(String) -> Option<T>, N: Fn(String) -> Option<T>>(
    target: &str,
    if_bash: Y,
    if_not_bash: N,
) -> Option<T> {
    let shell = env::var("SHELL").ok().unwrap_or_default();

    if shell.ends_with(target) {
        if_bash(shell)
    } else {
        if_not_bash(shell)
    }
}

/// Run a callback function if bash, or show a message saying the user's shell is unsupported.
pub fn run_if_bash_else_warn<T, Y: Fn(String) -> Option<T>>(if_bash: Y) -> Option<T> {
    run_if_shell("bash", if_bash, |shell| {
        eprintln!(
            "Unsupported shell '{}'. Currently, only bash is supported.",
            shell.blue()
        );
        None
    })
}

// todo: run_if_shell_async + run_if_bash_else_warn_async
