use crate::metadata::LoadMetadataConfig;
use crate::{
    animate::{show_loading_indicator, AnimationSettings},
    cli::{InjectOptions, Process},
    metadata::Metadata,
    uv::{uv, Helpers},
    venv::setup_environ_from_requirement,
};
use anyhow::Context;
use owo_colors::OwoColorize;

pub async fn inject_package(
    venv_spec: &str,
    to_inject_specs: &[String],
    no_cache: bool,
) -> anyhow::Result<String> {
    let (requirement, environ) = setup_environ_from_requirement(venv_spec).await?;
    let mut metadata = Metadata::for_requirement(&requirement, &LoadMetadataConfig::none()).await;

    let mut args = vec!["pip", "install"];

    if no_cache {
        args.push("--no-cache");
    }

    // vec<string> -> vec<str>
    let inject_args: Vec<&str> = to_inject_specs.iter().map(AsRef::as_ref).collect();
    args.extend(inject_args);

    let promise = uv(&args);

    let to_inject_str = &to_inject_specs.join(", ");
    show_loading_indicator(
        promise,
        format!("injecting {} into {}", &to_inject_str, &metadata.name),
        AnimationSettings::default(),
    )
    .await?;

    metadata
        .injected
        .extend(to_inject_specs.iter().map(ToString::to_string));

    metadata.save(&environ.to_path_buf()).await?;

    Ok(format!(
        "💉 Injected [{}] into {}.",
        &to_inject_str,
        &metadata.name.green(),
    ))
}

impl Process for InjectOptions {
    async fn process(self) -> anyhow::Result<i32> {
        match inject_package(&self.into, &self.package_specs, self.no_cache).await {
            Ok(msg) => {
                println!("{msg}");
                Ok(0)
            },
            Err(msg) => Err(msg).with_context(|| {
                format!(
                    "Something went wrong trying to inject {:?} into '{}';",
                    &self.package_specs, self.into
                )
            }),
        }
    }
}
