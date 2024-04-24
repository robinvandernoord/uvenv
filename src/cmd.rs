use crate::helpers::ResultToString;
use std::env;
use std::ffi::OsStr;
use std::path::PathBuf;
use std::process::Stdio;
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

    Some(binary)
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
                    Some(prefix) => Err(format!("{} | {}", prefix, err)),
                    None => Err(err),
                }
            },
        },
        Err(result) => {
            let err = result.to_string();
            match err_prefix {
                Some(prefix) => Err(format!("{} | {}", prefix, err)),
                None => Err(err),
            }
        },
    }
}
