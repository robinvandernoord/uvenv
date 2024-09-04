use crate::animate::{show_loading_indicator, AnimationSettings};
use crate::cmd::{run, run_get_output};
use anyhow::bail;
use pep508_rs::Requirement;
use serde::{Deserialize, Serialize};
use std::path::Path;
use std::str::FromStr;
use tempfile::NamedTempFile;

#[derive(Debug, Serialize, Deserialize)]
struct PipDownloadInfo {
    url: String,
    dir_info: serde_json::Value, // Since dir_info is an empty object, we can use serde_json::Value here
}

#[derive(Debug, Serialize, Deserialize)]
struct PipMetadata {
    metadata_version: String,
    name: String,
    version: String,
    classifier: Vec<String>,
    requires_dist: Vec<String>,
    requires_python: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct PipInstallItem {
    download_info: PipDownloadInfo,
    is_direct: bool,
    is_yanked: bool,
    requested: bool,
    requested_extras: Option<Vec<String>>,
    metadata: PipMetadata,
}

// #[derive(Debug, Serialize, Deserialize)]
// struct Environment {
//     implementation_name: String,
//     implementation_version: String,
//     os_name: String,
//     platform_machine: String,
//     platform_release: String,
//     platform_system: String,
//     platform_version: String,
//     python_full_version: String,
//     platform_python_implementation: String,
//     python_version: String,
//     sys_platform: String,
// }

#[derive(Debug, Serialize, Deserialize)]
struct PipData {
    version: String,
    pip_version: String,
    install: Vec<PipInstallItem>,
    // environment: Environment,
}

pub async fn pip(args: &[&str]) -> anyhow::Result<bool> {
    let err_prefix = format!("pip {}", args[0]);

    run("pip", args, Some(err_prefix)).await
}

#[derive(Debug)]
pub struct FakeInstallResult {
    pub name: String,
    pub file_url: String,
}

impl FakeInstallResult {
    pub fn to_spec(&self) -> String {
        format!("{} @ {}", self.name, self.file_url)
    }
}

pub async fn fake_install(install_spec: &str) -> anyhow::Result<FakeInstallResult> {
    let tempfile = NamedTempFile::new()?;
    let Some(tempfile_path) = tempfile.as_ref().to_str() else {
        bail!("No temp file could be created for a dry pip install.",)
    };

    // array instead of vec:
    let args = [
        "install",
        "--no-deps",
        "--dry-run",
        "--ignore-installed",
        "--report",
        tempfile_path,
        install_spec,
    ];

    pip(&args).await?;

    let json_file = tempfile.reopen()?;

    let pip_data: PipData = serde_json::from_reader(json_file)?;

    let Some(install) = pip_data.install.first() else {
        bail!("Failed to find package name for local install.",)
    };

    // if extras exist, the full name is name[extras]. Otherwise, it's just the name.
    let full_name = install.requested_extras.as_ref().map_or_else(
        || String::from(&install.metadata.name),
        |extras| format!("{}[{}]", &install.metadata.name, extras.join(",")),
    );

    let file_url = &install.download_info.url;

    Ok(FakeInstallResult {
        name: full_name,
        file_url: file_url.into(),
    })
}

pub async fn try_parse_local_requirement(
    install_spec: &str
) -> anyhow::Result<(Requirement, String)> {
    // fake install and extract the relevant info
    let promise = fake_install(install_spec);

    let result = show_loading_indicator(
        promise,
        format!("Trying to install local package {install_spec}"),
        AnimationSettings::default(),
    )
    .await?;

    let new_install_spec = result.to_spec();
    let requirement = Requirement::from_str(&new_install_spec)?;

    Ok((requirement, new_install_spec))
}

pub async fn parse_requirement(install_spec: &str) -> anyhow::Result<(Requirement, String)> {
    match Requirement::from_str(install_spec) {
        Ok(requirement) => Ok((requirement, String::from(install_spec))),
        Err(_) => try_parse_local_requirement(install_spec).await,
    }
}

pub async fn pip_freeze(python: &Path) -> anyhow::Result<String> {
    // uv pip freeze doesn't work for system
    run_get_output(python, &["-m", "pip", "freeze"]).await
}
