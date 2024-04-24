use crate::cli::{Process, SelfUpdateOptions};
use crate::commands::runpython::process_subprocess;
use crate::uv::find_sibling;
use owo_colors::OwoColorize;

pub async fn self_update(with_uv: bool) -> Result<i32, String> {
    let Some(exe) = find_sibling("python").await else {
        return Err(format!(
            "Python could not be found! Is `{}` installed globally (without a venv)?",
            "uvx".green()
        ));
    };

    // todo: with 'uv' instead of pip later?
    let mut args = vec!["-m", "pip", "install", "--upgrade", "uvx"];

    if with_uv {
        args.push("uv");
    }

    return process_subprocess(&exe, &args);
}

impl Process for SelfUpdateOptions {
    async fn process(self) -> Result<i32, String> {
        self_update(!self.without_uv).await
    }
}
