use pep440_rs::Version;
use pep508_rs::{PackageName, Requirement};
use rkyv::{de::deserializers::SharedDeserializeMap, Deserialize};
use uv_client::{RegistryClient, RegistryClientBuilder, SimpleMetadatum};

use crate::pip::parse_requirement;
use crate::uv::uv_cache;

pub async fn get_client() -> Option<RegistryClient> {
    let cache = uv_cache()?;

    Some(RegistryClientBuilder::new(cache).build())
}

pub fn deserialize_version(datum: &rkyv::Archived<Version>) -> Option<Version> {
    // for some reason, pycharm doesn't understand this type (but it compiles)
    let version: Option<Version> = datum.deserialize(&mut SharedDeserializeMap::new()).ok();
    version
}

#[allow(dead_code)]
pub fn deserialize_metadata(datum: &rkyv::Archived<SimpleMetadatum>) -> Option<SimpleMetadatum> {
    // for some reason, pycharm doesn't understand this type (but it compiles)
    let full: Option<SimpleMetadatum> = datum.deserialize(&mut SharedDeserializeMap::new()).ok();
    full
}

pub async fn get_latest_version_for_packagename(package_name: &PackageName) -> Option<Version> {
    let client = get_client().await?;

    let data = client.simple(package_name).await.ok()?;

    if let Some((_, metadata)) = data.iter().next_back() {
        if let Some(latest) = metadata.iter().next_back() {
            return deserialize_version(&latest.version);
        }
    }

    None
}
#[allow(dead_code)]
pub async fn get_pypi_data_for_packagename(package_name: &PackageName) -> Option<SimpleMetadatum> {
    let client = get_client().await?;

    let data = client.simple(package_name).await.ok()?;

    if let Some((_, metadata)) = data.iter().next_back() {
        if let Some(latest) = metadata.iter().next_back() {
            return deserialize_metadata(latest);
        }
    }

    None
}

pub async fn get_latest_version_for_requirement(req: &Requirement) -> Option<Version> {
    get_latest_version_for_packagename(&req.name).await
}
#[allow(dead_code)]
pub async fn get_pypi_data_for_requirement(req: &Requirement) -> Option<SimpleMetadatum> {
    get_pypi_data_for_packagename(&req.name).await
}

pub async fn get_latest_version(package_spec: &str) -> Option<Version> {
    let (requirement, _) = parse_requirement(package_spec).await.ok()?;

    get_latest_version_for_requirement(&requirement).await
}

#[allow(dead_code)]
pub async fn get_pypi_data(package_spec: &str) -> Option<SimpleMetadatum> {
    let (requirement, _) = parse_requirement(package_spec).await.ok()?;

    get_pypi_data_for_requirement(&requirement).await
}
