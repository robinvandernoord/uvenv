use owo_colors::OwoColorize;

use crate::{
    cli::{Process, ReinstallOptions},
    commands::{install::install_package, uninstall::uninstall_package},
    metadata::{venv_path, Metadata},
    pip::parse_requirement,
    uv::ExtractInfo,
};

pub async fn reinstall(
    install_spec: &str,
    python: Option<&String>,
    force: bool,
    with_injected: bool,
    no_cache: bool,
    editable: bool,
) -> Result<String, String> {
    let (requirement, _resolved_install_spec) = parse_requirement(install_spec).await?;
    let requirement_name = requirement.name.to_string();

    let venv_dir = venv_path(&requirement_name);

    if !venv_dir.exists() && !force {
        return Err(
            format!("'{}' was not previously installed. Please run 'uvx install {}' or pass `--force` instead.", 
            &requirement_name,
            &install_spec,
        )
        );
    }

    let current_metadata = Metadata::for_requirement(&requirement, false).await;

    let install_spec_changed =
        editable || !requirement.version().is_empty() || !requirement.extras().is_empty();

    if let Err(err) = uninstall_package(&requirement_name, force).await {
        eprintln!(
            "{}: something went wrong during uninstall ({})",
            "Warning".yellow(),
            err
        );
    }

    let new_install_spec = if install_spec_changed {
        install_spec
    } else {
        &current_metadata.install_spec
    };

    let inject = if with_injected {
        current_metadata.vec_injected()
    } else {
        Vec::new()
    };

    install_package(
        new_install_spec,
        None,
        python,
        force,
        inject,
        no_cache,
        editable,
    )
    .await
}

impl Process for ReinstallOptions {
    async fn process(self) -> Result<i32, String> {
        match reinstall(
            &self.package,
            self.python.as_ref(),
            self.force,
            !self.without_injected,
            self.no_cache,
            self.editable,
        )
        .await
        {
            Ok(msg) => {
                println!("{}", msg);
                Ok(0)
            },
            Err(msg) => Err(msg),
        }
    }
}
