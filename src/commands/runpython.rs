use std::ffi::OsStr;
use std::path::Path;
use subprocess::Exec;

use crate::{
    cli::{Process, RunpythonOptions},
    helpers::ResultToString,
    venv::setup_environ_from_requirement,
};

pub fn process_subprocess<S: AsRef<OsStr>>(
    exec_path: &Path,
    args: &[S],
) -> Result<i32, String> {
    Ok(
        match Exec::cmd(exec_path).args(args).join().map_err_to_string()? {
            subprocess::ExitStatus::Exited(int) => int as i32,
            subprocess::ExitStatus::Signaled(int) => int as i32,
            subprocess::ExitStatus::Other(int) => int,
            subprocess::ExitStatus::Undetermined => 0,
        },
    )
}

pub async fn run_python(
    venv_name: &str,
    python_args: Vec<String>,
) -> Result<i32, String> {
    let (_, environ) = setup_environ_from_requirement(venv_name).await?;

    let py = environ.interpreter().sys_executable();

    // Launch Python in interactive mode
    process_subprocess(py, &python_args)
}

impl Process for RunpythonOptions {
    async fn process(self) -> Result<i32, String> {
        match run_python(&self.venv, self.python_args).await {
            Ok(code) => Ok(code),
            Err(msg) => Err(msg),
        }
    }
}
