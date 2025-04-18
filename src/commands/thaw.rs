use crate::cli::{OutputFormat, Process, ThawOptions};
use crate::lockfile::AutoDeserialize;
use crate::lockfile::v0::LockfileV0;
use crate::lockfile::v1::LockfileV1;
use anyhow::{Context, bail};
use core::fmt::Debug;
use owo_colors::OwoColorize;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Default, Serialize, Deserialize)]
struct OnlyVersion {
    // only load version first. Other fields may change but this will remain the same.
    version: i8,
}

pub trait Thaw {
    async fn thaw(
        options: &ThawOptions,
        data: &[u8],
        format: OutputFormat,
    ) -> anyhow::Result<i32>
    where
        Self: Sized + Debug + DeserializeOwned;
}

async fn search_default_files() -> std::io::Result<Vec<u8>> {
    // 1. uvenv.lock
    // 2. uvenv.toml
    // 3. uvenv.json
    let possible_files = ["uvenv.lock", "uvenv.toml", "uvenv.json"];

    let mut last_err = None;
    for filename in possible_files {
        match tokio::fs::read(filename).await {
            Ok(contents) => return Ok(contents),
            Err(err) => last_err = Some(err),
        }
    }

    Err(last_err
        .unwrap_or_else(|| std::io::Error::new(std::io::ErrorKind::NotFound, "No lockfile found")))
}

impl Process for ThawOptions {
    async fn process(self) -> anyhow::Result<i32> {
        let maybe_contents = if self.filename.is_empty() {
            // try to find file instead:
            search_default_files().await
        } else {
            tokio::fs::read(&self.filename).await
        };

        let contents = maybe_contents.with_context(|| {
            format!(
                "Failed to determine lockfile version in {}",
                self.filename.red()
            )
        })?;

        if let Some((version, format)) = OnlyVersion::auto(&contents) {
            match version {
                OnlyVersion { version: 0 } => LockfileV0::thaw(&self, &contents, format).await,
                OnlyVersion { version: 1 } => LockfileV1::thaw(&self, &contents, format).await,
                OnlyVersion { .. } => {
                    bail!("Unsupported version!")
                },
            }
        } else {
            bail!("Could not determine filetype of {}.", self.filename.red());
        }
    }
}
