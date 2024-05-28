use crate::cli::{CreateOptions, Process};
use crate::metadata::{get_venv_dir, Metadata};
use crate::venv::{activate_venv, create_venv_raw};
use owo_colors::OwoColorize;

pub async fn create(
    name: &str,
    python: Option<&String>,
    seed: bool,
    force: bool,
) -> Result<String, String> {
    let venv_path = get_venv_dir().join(name);

    create_venv_raw(&venv_path, python, force, seed).await?;
    let venv = activate_venv(&venv_path).await?;

    let mut metadata = Metadata::new(name);
    // install spec should be empty to indicate bare create!
    metadata.install_spec = String::new();
    metadata.fill_python(&venv);

    metadata.save(&venv_path).await?;

    Ok(format!("🏗️ Succesfully created '{}'!", name.green()))
}

impl Process for CreateOptions {
    async fn process(self) -> Result<i32, String> {
        match create(
            &self.venv_name,
            self.python.as_ref(),
            !self.no_seed,
            self.force,
        )
        .await
        {
            Ok(msg) => {
                println!("{msg}");
                Ok(0)
            },
            Err(msg) => Err(msg),
        }
    }
}
