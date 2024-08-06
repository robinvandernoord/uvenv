use crate::cli::{Process, SelfChangelogOptions};
use anyhow::{anyhow, Context};
use owo_colors::OwoColorize;
use regex::Regex;
use std::collections::BTreeMap;

const CHANGELOG_URL: &str =
    "https://raw.githubusercontent.com/robinvandernoord/uvenv/uvenv/CHANGELOG.md";

type Changelogs = BTreeMap<String, BTreeMap<String, Vec<String>>>;

fn parse_changelog(markdown: &str) -> Changelogs {
    // BTreeMap is like a HashMap but ordered
    let mut changelog: BTreeMap<String, BTreeMap<String, Vec<String>>> = BTreeMap::new();
    let mut current_version = String::new();
    let mut current_category = String::new();

    let Ok(version_re) = Regex::new(r"^## v?(.+)") else {
        return changelog;
    };
    let Ok(category_re) = Regex::new(r"^### (.+)") else {
        return changelog;
    };
    let Ok(feature_re) = Regex::new(r"^[*-] (.+)") else {
        return changelog;
    };

    for line in markdown.lines() {
        if line.starts_with("# Changelog") {
            continue;
        }

        if let Some(version_caps) = version_re.captures(line) {
            let version = version_caps[1].to_string();
            changelog.insert(version.clone(), BTreeMap::new());
            current_version = version;
            continue;
        }

        if let Some(category_caps) = category_re.captures(line) {
            let category = category_caps[1].to_string();
            if let Some(map) = changelog.get_mut(&current_version) {
                map.insert(category.clone(), Vec::new());
                current_category = category;
            }
            continue;
        }

        if let Some(feature_caps) = feature_re.captures(line) {
            let features = feature_caps[1].to_string();

            if let Some(map) = changelog.get_mut(&current_version) {
                if let Some(vec) = map.get_mut(&current_category) {
                    vec.push(features);
                }
            }
            continue;
        }
    }

    changelog
}

async fn _get_changelog() -> reqwest::Result<String> {
    let resp = reqwest::get(CHANGELOG_URL).await?;
    let resp = resp.error_for_status()?;

    let body = resp.text().await?;

    Ok(body)
}

pub async fn get_changelog() -> anyhow::Result<String> {
    _get_changelog().await.map_err(|e| anyhow!(e)) // reqwest to anyhow
}

fn color(category: &str) -> String {
    match category.to_lowercase().trim() {
        "fix" | "fixes" => category.yellow().to_string(),
        "feature" | "feat" | "features" => category.green().to_string(),
        "documentation" | "docs" => category.blue().to_string(),
        "breaking change" | "break" => category.red().to_string(),
        _ => category.white().to_string(),
    }
}

fn colored_markdown(md: &str) -> String {
    let bold_re = Regex::new(r"(\*\*.*?\*\*)").expect("Failed to create bold regex");
    let mut final_string = String::new();

    for part in bold_re.split(md) {
        if part.starts_with("**") && part.ends_with("**") {
            let bold_text = part.trim_start_matches("**").trim_end_matches("**");
            final_string.push_str(&bold_text.bold().to_string());
        } else {
            final_string.push_str(part);
        }
    }

    final_string
}

pub fn display_changelog(changelog: &Changelogs) {
    for (version, changes) in changelog {
        println!("- {}", version.bold());
        for (category, descriptions) in changes {
            println!("-- {}", color(category));
            for change in descriptions {
                println!("---- {}", colored_markdown(change));
            }
        }
    }
}

pub async fn changelog() -> anyhow::Result<i32> {
    let md = get_changelog().await?;
    let parsed = parse_changelog(&md);

    display_changelog(&parsed);

    Ok(0)
}

impl Process for SelfChangelogOptions {
    async fn process(self) -> anyhow::Result<i32> {
        changelog()
            .await
            .with_context(|| "Something went wrong while loading the changelog;".to_string())
    }
}
