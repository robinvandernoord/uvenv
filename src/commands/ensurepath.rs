use anyhow::bail;
use std::path::PathBuf;

use chrono::Local;
use core::fmt::Write;
use owo_colors::OwoColorize;
use tokio::{fs::OpenOptions, io::AsyncWriteExt};

use crate::helpers::PathAsStr;
use crate::{
    cli::{EnsurepathOptions, Process},
    metadata::{ensure_bin_dir, get_home_dir},
};

pub fn now() -> String {
    let dt = Local::now();

    match dt.to_string().split_once('.') {
        None => String::new(),
        Some((datetime, _)) => datetime.to_owned(),
    }
}

pub async fn append(
    file_path: &PathBuf,
    text: &str,
) -> anyhow::Result<()> {
    let mut file = OpenOptions::new().append(true).open(file_path).await?;

    file.write_all(text.as_bytes()).await?;

    Ok(())
}

pub async fn add_to_bashrc(
    text: &str,
    with_comment: bool,
) -> anyhow::Result<()> {
    let path = get_home_dir().join(".bashrc");

    let now = now();
    let mut final_text = String::from("\n");
    if with_comment {
        // final_text.push_str(&format!("# Added by `uvenv` at {now}\n"));
        writeln!(final_text, "# Added by `uvenv` at {now}")?;
    }

    final_text.push_str(text);
    final_text.push('\n');

    append(&path, &final_text).await
}

pub async fn ensure_path(force: bool) -> anyhow::Result<()> {
    let bin_path = ensure_bin_dir().await;
    let bin_dir = bin_path.as_str();

    let path = std::env::var("PATH").unwrap_or_default();

    // let parts: Vec<&str> = path.split(':').collect();
    // if parts.contains(&bin_dir) && !force {
    if !force && path.split(':').any(|x| x == bin_dir) {
        bail!("{}: {} is already added to your path. Use '--force' to add it to your .bashrc file anyway.",
                "Warning".yellow(),
                bin_dir.green()
        );
    }

    add_to_bashrc(&format!("export PATH=\"$PATH:{bin_dir}\""), true).await?;

    println!("Added '{}' to ~/.bashrc", bin_dir.green());
    Ok(())
}

pub async fn ensure_path_generate() -> String {
    let bin_path = ensure_bin_dir().await;
    let bin_dir = bin_path.as_str();
    format!("export PATH=\"$PATH:{bin_dir}\"")
}

impl Process for EnsurepathOptions {
    async fn process(self) -> anyhow::Result<i32> {
        if let Err(msg) = ensure_path(self.force).await {
            Err(msg)
            // .with_context(|| format!("Something went wrong trying to ensure a proper PATH;"))
        } else {
            Ok(0)
        }
    }
}
