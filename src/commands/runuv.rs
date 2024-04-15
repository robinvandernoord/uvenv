use crate::{
    cli::{Process, RunuvOptions},
    uv::uv_with_output,
    venv::setup_environ_from_requirement,
};

pub async fn runuv(
    venv_name: &str,
    uv_args: Vec<String>,
) -> Result<String, String> {
    setup_environ_from_requirement(venv_name).await?;

    uv_with_output(uv_args).await?;

    return Ok(String::new());
}

impl Process for RunuvOptions {
    async fn process(self) -> Result<i32, String> {
        match runuv(&self.venv, self.uv_args).await {
            Ok(msg) => {
                println!("{}", msg);
                return Ok(0);
            },
            Err(msg) => return Err(msg),
        }
    }
}
