use std::env;
use std::ffi::OsStr;
use std::path::PathBuf;
use std::process::Stdio;

use owo_colors::OwoColorize;
use tokio::fs::canonicalize;
use tokio::process::Command;

use crate::helpers::ResultToString;

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

    if binary.exists() {
        Some(binary)
    } else {
        None
    }
}

pub async fn run_print_output<S1: AsRef<OsStr>, S2: AsRef<OsStr>>(
    command: S1,
    args: Vec<S2>,
) -> Result<i32, String> {
    let mut cmd = Command::new(command);
    cmd.args(args);
    cmd.stdout(Stdio::inherit());
    cmd.stderr(Stdio::inherit());
    let code = cmd.status().await.map_err_to_string()?;

    Ok(code.code().unwrap_or(-1))
}

pub async fn run_get_output<S1: AsRef<OsStr>, S2: AsRef<OsStr>>(
    command: S1,
    args: Vec<S2>,
) -> Result<String, String> {
    let result = Command::new(command).args(args).output().await;

    match result {
        Ok(result) => match result.status.code() {
            Some(0) => Ok(String::from_utf8(result.stdout).unwrap_or_default()),
            Some(_) | None => Err(String::from_utf8(result.stderr).unwrap_or_default()),
        },
        Err(result) => Err(result.to_string()),
    }
}

pub async fn run<S1: AsRef<OsStr>, S2: AsRef<OsStr>>(
    script: S1,
    args: Vec<S2>,
    err_prefix: Option<String>,
) -> Result<bool, String> {
    let result = Command::new(script).args(args).output().await;

    match result {
        Ok(result) => match result.status.code() {
            Some(0) => Ok(true),
            Some(_) | None => {
                let err = String::from_utf8(result.stderr).unwrap_or_default();
                match err_prefix {
                    Some(prefix) => Err(format!("{prefix} | {err}")),
                    None => Err(err),
                }
            },
        },
        Err(result) => {
            let err = result.to_string();
            match err_prefix {
                Some(prefix) => Err(format!("{prefix} | {err}")),
                None => Err(err),
            }
        },
    }
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
