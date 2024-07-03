use crate::{
    cli::{Process, RunuvOptions},
    uv::uv_with_output,
    venv::setup_environ_from_requirement,
};
use anyhow::anyhow;

pub async fn runuv(
    venv_name: &str,
    uv_args: Vec<String>,
) -> Result<i32, String> {
    setup_environ_from_requirement(venv_name).await?;

    uv_with_output(uv_args).await
}

impl Process for RunuvOptions {
    async fn process(self) -> anyhow::Result<i32> {
        match runuv(&self.venv, self.uv_args).await {
            Ok(code) => Ok(code),
            Err(msg) => Err(anyhow!(msg)),
        }
    }
}
