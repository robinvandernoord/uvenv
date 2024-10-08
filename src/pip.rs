use crate::animate::{show_loading_indicator, AnimationSettings};
use crate::cmd::{run, run_get_output};
use anyhow::bail;
use core::cmp::Ordering;
use core::str::FromStr;
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use std::path::Path;
use tempfile::NamedTempFile;
use uv_pep508::Requirement;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
#[expect(clippy::partial_pub_fields, reason = "Only the url should be public.")]
pub struct PipDownloadInfo {
    pub url: String,
    dir_info: serde_json::Value, // Since dir_info is an empty object, we can use serde_json::Value here
}

impl PartialOrd for PipDownloadInfo {
    fn partial_cmp(
        &self,
        other: &Self,
    ) -> Option<Ordering> {
        Some(String::cmp(&self.url, &other.url))
    }
}

impl Ord for PipDownloadInfo {
    fn cmp(
        &self,
        other: &Self,
    ) -> Ordering {
        String::cmp(&self.url, &other.url)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Default, Serialize, Deserialize)]
pub struct PipMetadata {
    pub metadata_version: String,
    pub name: String,
    pub version: String,
    pub classifier: Vec<String>,
    pub requires_dist: Vec<String>,
    pub requires_python: String,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Default, Serialize, Deserialize)]
pub struct PipInstallItem {
    pub download_info: PipDownloadInfo,
    pub is_direct: bool,
    pub is_yanked: bool,
    pub requested: bool,
    pub requested_extras: Option<Vec<String>>,
    pub metadata: PipMetadata,
}

// #[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Default,  Serialize, Deserialize)]
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

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Default, Serialize, Deserialize)]
pub struct PipData {
    pub version: String,
    pub pip_version: String,
    // environment: Environment,
    pub install: VecDeque<PipInstallItem>, // we want to get .first() but owned -> pop_first
}

pub async fn pip(args: &[&str]) -> anyhow::Result<bool> {
    // unwrap_or_default doesn't work on &&str :(
    let err_prefix = format!("pip {}", args.first().unwrap_or(&""));

    run("pip", args, Some(err_prefix)).await
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Default, Serialize, Deserialize)]
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

    let mut pip_data: PipData = serde_json::from_reader(json_file)?;

    let Some(install) = pip_data.install.pop_front() else {
        bail!("Failed to find package name for local install.",)
    };

    // if extras exist, the full name is name[extras]. Otherwise, it's just the name.
    let name = install.requested_extras.as_ref().map_or_else(
        || String::from(&install.metadata.name),
        |extras| format!("{}[{}]", &install.metadata.name, extras.join(",")),
    );

    let PipDownloadInfo { url: file_url, .. } = install.download_info;

    Ok(FakeInstallResult { name, file_url })
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
