use pep440_rs::{Version, VersionSpecifier};
use pep508_rs::{PackageName, Requirement};
use rkyv::{de::deserializers::SharedDeserializeMap, Deserialize};
use uv_client::{RegistryClient, RegistryClientBuilder, SimpleMetadatum};

use crate::pip::parse_requirement;
use crate::uv::uv_cache;

pub fn get_client() -> RegistryClient {
    let cache = uv_cache();

    RegistryClientBuilder::new(cache).build()
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

pub async fn get_versions_for_packagename(
    package_name: &PackageName,
    stable: bool,
    constraint: Option<VersionSpecifier>,
) -> Vec<Version> {
    let mut versions: Vec<Version> = vec![];

    let client = get_client();

    let data = match client.simple(package_name).await {
        Err(err) => {
            eprintln!("Something went wrong: {err};");
            return versions;
        },
        Ok(data) => data,
    };

    if let Some((_, metadata)) = data.iter().next_back() {
        versions = metadata
            .iter()
            .filter_map(|metadatum| deserialize_version(&metadatum.version))
            .collect();
    }

    if stable {
        versions.retain(|version| !version.any_prerelease());
    }

    if let Some(specifier) = constraint {
        versions.retain(|version| specifier.contains(version));
    }

    versions
}

pub async fn get_latest_version_for_packagename(
    package_name: &PackageName,
    stable: bool,
    constraint: Option<VersionSpecifier>,
) -> Option<Version> {
    let versions = get_versions_for_packagename(package_name, stable, constraint).await;

    versions.last().cloned()
}
#[allow(dead_code)]
pub async fn get_pypi_data_for_packagename(package_name: &PackageName) -> Option<SimpleMetadatum> {
    let client = get_client();

    let data = client.simple(package_name).await.ok()?;

    if let Some((_, metadata)) = data.iter().next_back() {
        if let Some(latest) = metadata.iter().next_back() {
            return deserialize_metadata(latest);
        }
    }

    None
}

pub async fn get_latest_version_for_requirement(
    req: &Requirement,
    stable: bool,
    constraint: Option<VersionSpecifier>,
) -> Option<Version> {
    get_latest_version_for_packagename(&req.name, stable, constraint).await
}

pub async fn get_latest_version(
    package_spec: &str,
    stable: bool,
    constraint: Option<VersionSpecifier>,
) -> Option<Version> {
    let (requirement, _) = parse_requirement(package_spec).await.ok()?;
    get_latest_version_for_requirement(&requirement, stable, constraint).await
}
