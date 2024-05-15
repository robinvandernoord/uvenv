use crate::cli::{CreateOptions, Process};
use crate::metadata::get_venv_dir;
use crate::venv::create_venv_raw;
use owo_colors::OwoColorize;

pub async fn create(
    name: &str,
    python: Option<&String>,
    seed: bool,
    force: bool,
) -> Result<i32, String> {
    let venv_path = get_venv_dir().join(name);

    create_venv_raw(&venv_path, python, force, seed).await?;

    println!("Succesfully created '{}'!", name.green());
    Ok(0)
}

impl Process for CreateOptions {
    async fn process(self) -> Result<i32, String> {
        create(
            &self.venv_name,
            self.python.as_ref(),
            !self.no_seed,
            self.force,
        )
        .await
    }
}
