use crate::{
    animate::{show_loading_indicator, AnimationSettings},
    cli::{InjectOptions, Process},
    metadata::Metadata,
    uv::{uv, Helpers},
    venv::setup_environ_from_requirement,
};
use owo_colors::OwoColorize;

pub async fn inject_package(
    venv_spec: &str,
    to_inject_specs: &[String],
    no_cache: bool,
) -> Result<String, String> {
    let (requirement, environ) = setup_environ_from_requirement(venv_spec).await?;
    let mut metadata = Metadata::for_requirement(&requirement, false).await;

    let mut args = vec!["pip", "install"];

    if no_cache {
        args.push("--no-cache")
    }

    // vec<string> -> vec<str>
    let inject_args: Vec<&str> = to_inject_specs.iter().map(|k| k.as_ref()).collect();
    args.extend(inject_args);

    let promise = uv(args);

    let to_inject_str = &to_inject_specs.join(", ");
    show_loading_indicator(
        promise,
        format!("injecting {} into {}", &to_inject_str, &metadata.name),
        AnimationSettings::default(),
    )
    .await?;

    metadata
        .injected
        .extend(to_inject_specs.iter().map(|k| k.to_string()));
    metadata.save(&environ.to_path_buf()).await?;

    Ok(format!(
        "💉 Injected [{}] into {}.",
        &to_inject_str,
        &metadata.name.green(),
    ))
}

impl Process for InjectOptions {
    async fn process(self) -> Result<i32, String> {
        match inject_package(&self.into, &self.package_specs, self.no_cache).await {
            Ok(msg) => {
                println!("{}", msg);
                Ok(0)
            },
            Err(msg) => Err(msg),
        }
    }
}
