use crate::{
    cli::{Process, RunpipOptions},
    cmd::run_print_output,
    venv::{setup_environ_from_requirement, venv_script},
};
use anyhow::Context;

pub async fn runpip(
    venv_name: &str,
    pip_args: &[String],
) -> anyhow::Result<i32> {
    let (_, env) = setup_environ_from_requirement(venv_name).await?;

    let script = venv_script(&env, "pip");

    run_print_output(script, pip_args).await
}

impl Process for RunpipOptions {
    async fn process(self) -> anyhow::Result<i32> {
        match runpip(&self.venv, &self.pip_args).await {
            Ok(code) => Ok(code),
            Err(msg) => Err(msg).with_context(|| {
                format!(
                    "Something went wrong trying to run pip in '{}';",
                    &self.venv
                )
            }),
        }
    }
}
