use std::path::PathBuf;

use chrono::Local;
use owo_colors::OwoColorize;
use tokio::{fs::OpenOptions, io::AsyncWriteExt};

use crate::{
    cli::{EnsurepathOptions, Process},
    helpers::ResultToString,
    metadata::{get_bin_dir, get_home_dir},
};

pub fn now() -> String {
    let dt = Local::now();

    match dt.to_string().split_once('.') {
        None => String::new(),
        Some((datetime, _)) => datetime.to_string(),
    }
}

pub async fn append(
    file: &PathBuf,
    text: String,
) -> Result<(), String> {
    let mut file = OpenOptions::new()
        .append(true)
        .open(file)
        .await
        .map_err_to_string()?;

    file.write_all(text.as_bytes()).await.map_err_to_string()?;

    Ok(())
}

pub async fn add_to_bashrc(
    text: &str,
    with_comment: bool,
) -> Result<(), String> {
    /*    with (Path.home() / ".bashrc").resolve().open("a") as f:
           now = str(datetime.now()).split(".")[0]
           final_text = "\n"
           final_text += f"# Added by `uvx` at {now}\n" if with_comment else ""
           final_text += text + "\n"
           f.write(final_text)
    */
    let path = get_home_dir().join(".bashrc");

    let now = now();
    let mut final_text = String::from("\n");
    if with_comment {
        final_text.push_str(&format!("# Added by `uvx` at {now}\n"))
    }

    final_text.push_str(text);
    final_text.push('\n');

    append(&path, final_text).await
}

pub async fn ensure_path(force: bool) -> Result<(), String> {
    let bin_path = get_bin_dir();
    let bin_dir = bin_path.to_str().unwrap_or_default();

    let path = std::env::var("PATH").unwrap_or_default();

    let parts: Vec<&str> = path.split(':').collect();

    if parts.contains(&bin_dir) && !force {
        return Err(format!("{}: {} is already added to your path. Use '--force' to add it to your .bashrc file anyway.",
            "Warning".yellow(),
            bin_dir.green()
    ))  ;
    }

    add_to_bashrc(&format!("export PATH=\"$PATH:{}\"", bin_dir), true).await?;

    println!("Added '{}' to ~/.bashrc", bin_dir.green());
    Ok(())
}

impl Process for EnsurepathOptions {
    async fn process(self) -> Result<i32, String> {
        ensure_path(self.force).await?;
        Ok(0)
    }
}
