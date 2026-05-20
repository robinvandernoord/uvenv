use crate::cli::{FreezeOptions, OutputFormat};
use crate::metadata::{Metadata, atomic_write};
use anyhow::anyhow;
use core::fmt::Debug;
use owo_colors::OwoColorize;
use regex::Regex;
use serde::Serialize;
use serde::de::DeserializeOwned;
use std::collections::BTreeMap;

pub mod v0;
pub mod v1;

type PackageMap<P> = BTreeMap<String, P>;

trait PackageSpec: From<Metadata> {}

trait Lockfile<'de, P: PackageSpec + From<Metadata> + Debug + Serialize>:
    Sized + Debug + Serialize
{
    fn new(packages: PackageMap<P>) -> Self;

    async fn serialize_and_patch(
        &self,
        options: &FreezeOptions,
    ) -> anyhow::Result<Vec<u8>>;

    // predefined implementations:

    async fn dump_to_file(
        &self,
        options: &FreezeOptions,
    ) -> anyhow::Result<()> {
        let format = &options.format;
        let filename = &options.filename;

        let serialized = self.serialize_and_patch(options).await?;

        atomic_write(filename, &serialized).await?;

        eprintln!(
            "Saved {} to {}.",
            format.to_string().blue(),
            filename.green()
        );

        Ok(())
    }

    async fn write(
        packages: PackageMap<P>,
        options: &FreezeOptions,
    ) -> anyhow::Result<bool> {
        let instance = Self::new(packages);
        instance.dump_to_file(options).await?;
        Ok(true)
    }
}

pub trait AutoDeserialize: DeserializeOwned {
    fn from_json(data: &[u8]) -> anyhow::Result<Self> {
        serde_json::from_slice(data).map_err(|err| anyhow!(err))
    }
    fn from_msgpack(data: &[u8]) -> anyhow::Result<Self> {
        rmp_serde::decode::from_slice(data).map_err(|err| anyhow!(err))
    }
    fn from_toml(data: &[u8]) -> anyhow::Result<Self> {
        let data_str = String::from_utf8(data.to_owned())?;
        toml::from_str(&data_str).map_err(|err| anyhow!(err))
    }

    fn from_format(
        data: &[u8],
        format: OutputFormat,
    ) -> anyhow::Result<Self> {
        match format {
            OutputFormat::JSON => Self::from_json(data),
            OutputFormat::TOML => Self::from_toml(data),
            OutputFormat::Binary => Self::from_msgpack(data),
        }
    }

    fn auto(data: &[u8]) -> Option<(Self, OutputFormat)> {
        None /* Start with None so the rest or_else are all the same structure */
            .or_else(|| {
                Self::from_json(data)
                    .ok()
                    .map(|version| (version, OutputFormat::JSON))
            })
            .or_else(|| {
                Self::from_msgpack(data)
                    .ok()
                    .map(|version| (version, OutputFormat::Binary))
            })
            .or_else(|| {
                Self::from_toml(data)
                    .ok()
                    .map(|version| (version, OutputFormat::TOML))
            })
    }
}

fn extract_python_version(input: &str) -> Option<String> {
    let Ok(re) = Regex::new(r"(\d+)\.(\d+)") else {
        return None;
    };

    re.captures(input).map(|caps| {
        let major = &caps[1];
        let minor = &caps[2];
        format!("{major}.{minor}")
    })
}

impl<L: DeserializeOwned> AutoDeserialize for L {}
