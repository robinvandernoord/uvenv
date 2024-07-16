use crate::animate::{show_loading_indicator, AnimationSettings};
use crate::cli::{Process, UnInjectOptions};
use crate::metadata::{LoadMetadataConfig, Metadata};
use crate::venv::setup_environ_from_requirement;
use anyhow::anyhow;
use itertools::Itertools;
use owo_colors::OwoColorize;

use crate::uv::{uv, Helpers};

pub async fn eject_package(
    from: &str,
    to_eject_specs: &[String], // .contains is used, so String is required.
) -> anyhow::Result<String> {
    let (requirement, environ) = setup_environ_from_requirement(from).await?;
    let mut metadata = Metadata::for_requirement(&requirement, &LoadMetadataConfig::none()).await;

    let mut args = vec!["pip", "uninstall"];

    let eject_args: Vec<&str> = to_eject_specs.iter().map(AsRef::as_ref).collect();
    args.extend(eject_args);

    let promise = uv(&args);

    let to_eject_str = &to_eject_specs.iter().map(|it| it.green()).join(", ");
    show_loading_indicator(
        promise,
        format!("injecting {} into {}", &to_eject_str, &metadata.name),
        AnimationSettings::default(),
    )
    .await?;

    metadata.injected = metadata
        .injected
        .iter()
        .filter(|i| !to_eject_specs.contains(i))
        .map(ToString::to_string)
        .collect();

    metadata.save(&environ.to_path_buf()).await?;

    Ok(format!(
        "⏏️  Ejected [{}] from {}.",
        &to_eject_str,
        &metadata.name.green(),
    ))
}

impl Process for UnInjectOptions {
    async fn process(self) -> anyhow::Result<i32> {
        match eject_package(&self.outof, &self.package_specs).await {
            Ok(msg) => {
                println!("{msg}");
                Ok(0)
            },
            Err(msg) => Err(anyhow!(msg)),
        }
    }
}
