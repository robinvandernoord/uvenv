use subprocess::Exec;

use crate::{
    cli::{Process, RunpythonOptions},
    helpers::ResultToString,
    venv::setup_environ_from_requirement,
};

pub async fn run_python(
    venv_name: &str,
    python_args: Vec<String>,
) -> Result<i32, String> {
    setup_environ_from_requirement(venv_name).await?;

    // Launch Python in interactive mode
    Ok(
        match Exec::cmd("python")
            .args(&python_args)
            .join()
            .map_err_to_string()?
        {
            subprocess::ExitStatus::Exited(int) => int as i32,
            subprocess::ExitStatus::Signaled(int) => int as i32,
            subprocess::ExitStatus::Other(int) => int,
            subprocess::ExitStatus::Undetermined => 0,
        },
    )
}

impl Process for RunpythonOptions {
    async fn process(self) -> Result<i32, String> {
        return match run_python(&self.venv, self.python_args).await {
            Ok(code) => Ok(code),
            Err(msg) => Err(msg),
        };
    }
}
