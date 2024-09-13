use anyhow::Context;
use std::ffi::OsStr;
use std::path::Path;
use subprocess::Exec;

use crate::{
    cli::{Process, RunpythonOptions},
    venv::setup_environ_from_requirement,
};

#[expect(
    clippy::cast_lossless,
    clippy::as_conversions,
    reason = "The numbers wont be that big."
)]
pub fn process_subprocess<S: AsRef<OsStr>>(
    exec_path: &Path,
    args: &[S],
) -> anyhow::Result<i32> {
    Ok(match Exec::cmd(exec_path).args(args).join()? {
        subprocess::ExitStatus::Exited(int) => int as i32,
        subprocess::ExitStatus::Signaled(int) => int as i32,
        subprocess::ExitStatus::Other(int) => int,
        subprocess::ExitStatus::Undetermined => 0,
    })
}

pub async fn run_python(
    venv_name: &str,
    python_args: &[String],
) -> anyhow::Result<i32> {
    let (_, environ) = setup_environ_from_requirement(venv_name).await?;

    let py = environ.interpreter().sys_executable();

    // Launch Python in interactive mode
    process_subprocess(py, python_args)
}

impl Process for RunpythonOptions {
    async fn process(self) -> anyhow::Result<i32> {
        run_python(&self.venv, &self.python_args)
            .await
            .with_context(|| {
                format!(
                    "Something went wrong trying to run Python in '{}';",
                    &self.venv
                )
            })
    }
}
