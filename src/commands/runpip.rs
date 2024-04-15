use crate::{
    cli::{Process, RunpipOptions},
    uv::run_with_output,
    venv::{setup_environ_from_requirement, venv_script},
};

pub async fn runpip(
    venv_name: &str,
    pip_args: Vec<String>,
) -> Result<String, String> {
    let (_, env) = setup_environ_from_requirement(venv_name).await?;

    let script = venv_script(&env, "pip");

    run_with_output(script, pip_args).await?;

    return Ok(String::new());
}

impl Process for RunpipOptions {
    async fn process(self) -> Result<i32, String> {
        match runpip(&self.venv, self.pip_args).await {
            Ok(msg) => {
                println!("{}", msg);
                return Ok(0);
            },
            Err(msg) => return Err(msg),
        }
    }
}
