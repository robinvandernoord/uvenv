use crate::{
    cli::{Process, RunuvOptions},
    uv::uv_with_output,
    venv::setup_environ_from_requirement,
};
use anyhow::Context;

pub async fn runuv(
    venv_name: &str,
    uv_args: &[String],
) -> anyhow::Result<i32> {
    setup_environ_from_requirement(venv_name).await?;

    uv_with_output(uv_args).await
}

impl Process for RunuvOptions {
    async fn process(self) -> anyhow::Result<i32> {
        match runuv(&self.venv, &self.uv_args).await {
            Ok(code) => Ok(code),
            Err(msg) => Err(msg).with_context(|| {
                format!("Something went wrong trying to run uv in '{}';", &self.venv)
            }),
        }
    }
}
