use crate::pip::parse_requirement;
use crate::uv::uv_cache;
use rkyv::{deserialize, Archive, Archived, Deserialize};
use uv_pep440::{Version, VersionSpecifier};
use uv_pep508::{PackageName, Requirement};
use uv_pypi_types::Yanked;

use rkyv::api::high::HighDeserializer;
use std::collections::HashSet;
use uv_client::{
    OwnedArchive, RegistryClient, RegistryClientBuilder, SimpleMetadata, SimpleMetadatum,
    VersionFiles,
};
use uv_distribution_types::{IndexCapabilities, IndexUrl};

/// Shadow `RegistryClient` to hide new complexity of .simple
struct SimplePypi(RegistryClient);

impl SimplePypi {
    /// Use RegistryClient.simple to lookup a package on default package index
    async fn lookup<'index>(
        &'index self,
        package_name: &PackageName,
    ) -> anyhow::Result<Vec<(&'index IndexUrl, OwnedArchive<SimpleMetadata>)>> {
        let res = self
            .0
            .simple(package_name, None, &IndexCapabilities::default())
            .await?;

        Ok(res)
    }
}

impl Default for SimplePypi {
    /// Create a (default) Registry
    fn default() -> Self {
        let cache = uv_cache();
        let inner = RegistryClientBuilder::new(cache).build();

        Self(inner)
    }
}

/// usage: e.g. `let x: Option<VersionFiles> = deserialize(&metadatum.files);`
/// Note: pycharm will probably complain, but it WILL work for `ArchivedSimpleMetadatum`!
pub fn rkyv_deserialize<T>(archived: &Archived<T>) -> Option<T>
where
    T: Archive,
    T::Archived: Deserialize<T, HighDeserializer<rkyv::rancor::Error>>,
{
    deserialize(archived).ok()
}

fn deserialize_metadata(datum: &Archived<SimpleMetadatum>) -> Option<SimpleMetadatum> {
    // for some reason, pycharm doesn't understand this type (but it compiles)
    let full: Option<SimpleMetadatum> = rkyv_deserialize(datum);
    full
}

const fn is_yanked(yanked: &Option<Yanked>) -> bool {
    match yanked {
        None => false,
        Some(Yanked::Reason(_)) => true,
        Some(Yanked::Bool(bool)) => *bool,
    }
}

fn find_non_yanked_versions(metadata: &OwnedArchive<SimpleMetadata>) -> HashSet<Version> {
    let files_data: Vec<VersionFiles> = metadata
        .iter()
        .filter_map(|metadatum| rkyv_deserialize(&metadatum.files))
        .collect();

    let mut valid_versions = HashSet::new();

    for file in files_data {
        for source_dist in file.source_dists {
            if !is_yanked(&source_dist.file.yanked) {
                valid_versions.insert(source_dist.name.version);
            }
        }
        for wheel in file.wheels {
            if !is_yanked(&wheel.file.yanked) {
                valid_versions.insert(wheel.name.version);
            }
        }
    }

    valid_versions
}

pub async fn get_versions_for_packagename(
    package_name: &PackageName,
    stable: bool,
    constraint: Option<VersionSpecifier>,
) -> Vec<Version> {
    let mut versions: Vec<Version> = vec![];

    let client = SimplePypi::default();

    let data = match client.lookup(package_name).await {
        Err(err) => {
            eprintln!("Something went wrong: {err};");
            return versions;
        },
        Ok(data) => data,
    };

    if let Some((_, metadata)) = data.iter().next_back() {
        let not_yanked = find_non_yanked_versions(metadata);

        versions = metadata
            .iter()
            .filter_map(|metadatum| {
                rkyv_deserialize(&metadatum.version).filter(|version| not_yanked.contains(version))
            })
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
#[expect(
    dead_code,
    reason = "More generic than the used code above (which only looks at version info)"
)]
pub async fn get_pypi_data_for_packagename(package_name: &PackageName) -> Option<SimpleMetadatum> {
    let client = SimplePypi::default();

    let data = client.lookup(package_name).await.ok()?;

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
