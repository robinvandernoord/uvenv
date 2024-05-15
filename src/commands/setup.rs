use owo_colors::OwoColorize;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

use crate::cli::{Process, SetupOptions};
use crate::cmd::run_if_bash_else_warn;
use crate::commands::activate::install_activate;
use crate::commands::completions::completions;
use crate::commands::ensurepath::ensure_path;
use crate::metadata::{get_work_dir, load_generic_msgpack, store_generic_msgpack};

#[derive(Debug, PartialEq, Deserialize, Serialize)] // dbg_pls::DebugPls
pub struct SetupMetadata {
    // order is important, new features should go last!!
    #[serde(default)]
    pub feature_ensurepath: bool,
    #[serde(default)]
    pub feature_completions: bool,
    #[serde(default)]
    pub feature_activate: bool,
}

impl SetupMetadata {
    pub fn new() -> Self {
        SetupMetadata {
            feature_ensurepath: false,
            feature_completions: false,
            feature_activate: false,
        }
    }
}

fn setup_metadata_filename() -> PathBuf {
    let workdir = get_work_dir();

    workdir.join("setup.metadata")
}

async fn _load_setup_metadata() -> Result<SetupMetadata, String> {
    let filename = setup_metadata_filename();

    let mut buf = Vec::new(); // allocate memory for the object

    // Open the msgpack file
    let metadata: SetupMetadata = load_generic_msgpack(&filename, &mut buf).await?;

    Ok(metadata)
}

pub async fn load_setup_metadata() -> SetupMetadata {
    _load_setup_metadata().await.unwrap_or(SetupMetadata::new())
}

pub async fn store_setup_metadata(metadata: &SetupMetadata) -> Result<(), String> {
    let filename = setup_metadata_filename();

    store_generic_msgpack(&filename, metadata).await
}

pub async fn setup_for_bash(
    do_ensurepath: bool,
    do_completions: bool,
    do_activate: bool,
    force: bool,
) -> Result<i32, String> {
    let mut any_warnings: bool = false;

    let mut metadata = load_setup_metadata().await;

    if do_ensurepath && (!metadata.feature_ensurepath || force) {
        if let Err(msg) = ensure_path(force).await {
            any_warnings = true;
            eprintln!("{}", msg);
        }
        metadata.feature_ensurepath = true;
    }

    if do_completions && (!metadata.feature_completions || force) {
        if let Err(msg) = completions(true).await {
            any_warnings = true;
            eprintln!("{}", msg);
        }
        metadata.feature_completions = true;
    }

    if do_activate && (!metadata.feature_activate || force) {
        if let Err(msg) = install_activate().await {
            any_warnings = true;
            eprintln!("{}", msg);
        }
        metadata.feature_activate = true;
    }

    if let Err(msg) = store_setup_metadata(&metadata).await {
        any_warnings = true;
        eprintln!("{}", msg);
    }

    println!("Setup finished, you may want to run `{}` now in order to apply these changes to your shell.", "exec bash".green());
    Ok(if any_warnings { 1 } else { 0 })
}

impl Process for SetupOptions {
    async fn process(self) -> Result<i32, String> {
        let result = run_if_bash_else_warn(move |_| {
            // some logic here
            let result = setup_for_bash(
                !self.skip_ensurepath,
                !self.skip_completions,
                !self.skip_activate,
                self.force,
            );

            // async is not possible in this block,
            // creating a run_if_bash_else_warn_async is non-trivial
            Some(result) // so just return a promise
        });

        match result {
            Some(promise) => {
                // finally, we can await
                promise.await
            },
            None => {
                // unsupported shell ->
                Ok(126)
            },
        }
    }
}
