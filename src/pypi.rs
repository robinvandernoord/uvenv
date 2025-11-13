use crate::pip::parse_requirement;
use crate::uv::uv_cache;
use uv_normalize::PackageName;
use uv_pep440::{Version, VersionSpecifier};
use uv_pep508::Requirement;
use uv_pypi_types::Yanked;

use std::collections::HashSet;
use tokio::sync::Semaphore;
use uv_client::{
    BaseClientBuilder, MetadataFormat, OwnedArchive, RegistryClient, RegistryClientBuilder,
    SimpleDetailMetadata,
};
use uv_distribution_types::IndexCapabilities;

/// Shadow `RegistryClient` to hide new complexity of .simple
struct SimplePypi(RegistryClient);

impl SimplePypi {
    /// Use `RegistryClient.package_metadata` to lookup a package on default package index
    async fn lookup(
        &self,
        package_name: &PackageName,
    ) -> anyhow::Result<Vec<OwnedArchive<SimpleDetailMetadata>>> {
        // 1 permit is sufficient
        let download_concurrency = Semaphore::new(1);

        let response = self
            .0
            .simple_detail(
                package_name,
                None,
                &IndexCapabilities::default(),
                &download_concurrency,
            )
            .await?;

        let mapped: Vec<_> = response
            .into_iter()
            .filter_map(|(_url, metadata)| match metadata {
                MetadataFormat::Simple(data) => Some(data),
                MetadataFormat::Flat(_) => None,
            })
            .collect();

        Ok(mapped)
    }
}

impl Default for SimplePypi {
    /// Create a (default) Registry
    fn default() -> Self {
        let cache = uv_cache();
        let base_client = BaseClientBuilder::default();
        let inner = RegistryClientBuilder::new(base_client, cache).build();

        Self(inner)
    }
}

#[expect(
    clippy::borrowed_box,
    reason = "If we remove the Box<> then Rust complains that we pass in the wrong type"
)]
fn is_yanked(maybe_yanked_box: Option<&Box<Yanked>>) -> bool {
    if let Some(yanked_box) = maybe_yanked_box.as_ref()
        && yanked_box.is_yanked()
    {
        true
    } else {
        false
    }
}

fn find_non_yanked_versions(metadata: &SimpleDetailMetadata) -> HashSet<&Version> {
    let mut valid_versions = HashSet::new();

    for metadatum in metadata.iter() {
        for source_dist in &metadatum.files.source_dists {
            if !is_yanked(source_dist.file.yanked.as_ref()) {
                valid_versions.insert(&source_dist.name.version);
            }
        }

        for wheel in &metadatum.files.wheels {
            if !is_yanked(wheel.file.yanked.as_ref()) {
                valid_versions.insert(&wheel.name.version);
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

    if let Some(metadata_archived) = data.iter().next_back() {
        let metadata = OwnedArchive::deserialize(metadata_archived);
        let not_yanked = find_non_yanked_versions(&metadata);

        versions = metadata
            .iter()
            .filter_map(|metadatum| {
                let version = metadatum.version.clone();

                not_yanked.contains(&version).then_some(version)
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
pub async fn get_pypi_data_for_packagename(
    package_name: &PackageName
) -> Option<SimpleDetailMetadata> {
    let client = SimplePypi::default();

    let data = client.lookup(package_name).await.ok()?;

    data.iter().next_back().map_or_else(
        || None,
        |metadata_archived| {
            let metadata = OwnedArchive::deserialize(metadata_archived);
            Some(metadata)
        },
    )
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
