use std::path::Path;
use std::str::FromStr;

use pep508_rs::Requirement;
use serde::{Deserialize, Serialize};
use tempfile::NamedTempFile;

use crate::cmd::{run, run_get_output};
use crate::{
    animate::{show_loading_indicator, AnimationSettings},
    helpers::ResultToString,
};

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

pub async fn pip(args: Vec<&str>) -> Result<bool, String> {
    let err_prefix = format!("pip {}", &args[0]);

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

pub async fn fake_install(install_spec: &str) -> Result<FakeInstallResult, String> {
    let mut args: Vec<&str> = vec![
        "install",
        "--no-deps",
        "--dry-run",
        "--ignore-installed",
        "--report",
    ];

    let tempfile = NamedTempFile::new().map_err_to_string()?;
    let Some(tempfile_path) = tempfile.as_ref().to_str() else {
        return Err(String::from(
            "No temp file could be created for a dry pip install.",
        ));
    };

    args.push(tempfile_path); // tmpfile
    args.push(install_spec); // tmpfile

    pip(args).await?;

    let json_file = tempfile.reopen().map_err_to_string()?;

    let pip_data: PipData = serde_json::from_reader(json_file).map_err_to_string()?;

    let Some(install) = pip_data.install.first() else {
        return Err(String::from(
            "Failed to find package name for local install.",
        ));
    };

    // let empty_vec = Vec::new();
    // let extras = match &install.requested_extras {
    //     Some(extras) => extras,
    //     None => &empty_vec,
    // };

    let full_name = match &install.requested_extras {
        Some(extras) => {
            format!("{}[{}]", &install.metadata.name, extras.join(","))
        },
        None => String::from(&install.metadata.name),
    };

    let file_url = &install.download_info.url;

    Ok(FakeInstallResult {
        name: full_name,
        file_url: String::from(file_url),
    })
}

pub async fn try_parse_local_requirement(
    install_spec: &str
) -> Result<(Requirement, String), String> {
    // fake install and extract the relevant info
    let promise = fake_install(install_spec);

    let result = show_loading_indicator(
        promise,
        format!("Trying to install local package {}", install_spec),
        AnimationSettings::default(),
    )
    .await?;

    let new_install_spec = result.to_spec();
    let requirement = Requirement::from_str(&new_install_spec).map_err_to_string()?;

    Ok((requirement, new_install_spec))
}

pub async fn parse_requirement(install_spec: &str) -> Result<(Requirement, String), String> {
    match Requirement::from_str(install_spec) {
        Ok(requirement) => Ok((requirement, String::from(install_spec))),
        Err(_) => try_parse_local_requirement(install_spec).await,
    }
}

pub async fn pip_freeze(python: &Path) -> Result<String, String> {
    // let py = python.to_str().unwrap_or_default(); // idk why python.to_string() doesn't work
    run_get_output(python, vec!["-m", "pip", "freeze"]).await
}
