use crate::{
    cli::{Process, RunpipOptions},
    uv::run_with_output,
    venv::{setup_environ_from_requirement, venv_script},
};

pub async fn runpip(
    venv_name: &str,
    pip_args: Vec<String>,
) -> Result<i32, String> {
    let (_, env) = setup_environ_from_requirement(venv_name).await?;

    let script = venv_script(&env, "pip");

    return run_with_output(script, pip_args).await;
}

impl Process for RunpipOptions {
    async fn process(self) -> Result<i32, String> {
        return match runpip(&self.venv, self.pip_args).await {
            Ok(code) => Ok(code),
            Err(msg) => Err(msg),
        };
    }
}
