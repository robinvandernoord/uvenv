/// Barebones placeholder to be extended in later versions.
use crate::cli::{FreezeOptions, OutputFormat, ThawOptions};
use crate::commands::freeze::Freeze;
use crate::commands::thaw::Thaw;
use crate::lockfile::{Lockfile, PackageMap, PackageSpec};
use crate::metadata::{Metadata, serialize_msgpack};
use core::fmt::Debug;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Default, Serialize, Deserialize)]
struct PackageSpecV0;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Default, Serialize, Deserialize)]
pub struct LockfileV0 {
    version: i8,
}

impl From<Metadata> for PackageSpecV0 {
    fn from(_: Metadata) -> Self {
        Self {}
    }
}

impl PackageSpec for PackageSpecV0 {}

impl Lockfile<'_, PackageSpecV0> for LockfileV0 {
    fn new(_: PackageMap<PackageSpecV0>) -> Self {
        Self { version: 0 }
    }

    async fn serialize_and_patch(
        &self,
        options: &FreezeOptions,
    ) -> anyhow::Result<Vec<u8>> {
        Ok(match options.format {
            OutputFormat::TOML => toml::to_string(self)?.into_bytes(),
            OutputFormat::JSON => serde_json::to_string_pretty(self)?.into_bytes(),
            OutputFormat::Binary => serialize_msgpack(self).await?,
        })
    }
}

impl Freeze for LockfileV0 {
    async fn freeze(options: &FreezeOptions) -> anyhow::Result<i32>
    where
        Self: Sized + Debug + Serialize,
    {
        let packages = PackageMap::new();
        Ok(Self::write(packages, options).await?.into())
    }
}

impl Thaw for LockfileV0 {
    async fn thaw(
        _options: &ThawOptions,
        _data: &[u8],
        _format: OutputFormat,
    ) -> anyhow::Result<i32>
    where
        Self: Sized + Debug + DeserializeOwned,
    {
        Ok(0)
    }
}
