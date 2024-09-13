use owo_colors::OwoColorize;
use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::cli::{CheckOptions, Process};
use crate::commands::list::list_packages;
use crate::metadata::LoadMetadataConfig;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Default, Serialize, Deserialize)]
struct Issues<'metadata> {
    #[serde(borrow)]
    outdated: Vec<&'metadata str>,
    #[serde(borrow)]
    scripts: BTreeMap<&'metadata str, Vec<String>>,
}

impl<'metadata> Issues<'metadata> {
    pub const fn new() -> Self {
        Self {
            outdated: Vec::new(),
            scripts: BTreeMap::new(),
        }
    }

    #[expect(clippy::as_conversions, reason = "The number won't be that big")]
    pub fn count_outdated(&self) -> i32 {
        self.outdated.len() as i32
    }
    #[expect(clippy::as_conversions, reason = "The number won't be that big")]
    pub fn count_scripts(&self) -> i32 {
        self.scripts
            .values()
            .fold(0, |acc, vec| acc + vec.len() as i32)
    }

    pub fn count(&self) -> i32 {
        self.count_outdated() + self.count_scripts()
    }

    pub fn print_json(&self) -> anyhow::Result<i32> {
        let json = serde_json::to_string_pretty(self)?;

        eprintln!("{json}");

        Ok(self.count())
    }

    fn print_human(&self) -> i32 {
        let issue_count = self.count();

        if issue_count == 0 {
            println!("{}", "✅ No issues found. Everything is up-to-date and all scripts are properly installed!".green().bold());
            return 0;
        }

        println!("{}", "🚨 Issues Overview:".bold().underline());

        // Display outdated issues
        if !self.outdated.is_empty() {
            println!("{}", "\n🔶 Outdated:".bold().yellow());
            for issue in &self.outdated {
                println!("  - {}", issue.red());
            }

            println!(
                "{}",
                "💡 Tip: you can use `uvenv upgrade <package>` to update a specific environment."
                    .blue()
            );
        }

        // Display script issues
        if !self.scripts.is_empty() {
            println!("{}", "\n🔶 Missing Scripts:".bold().yellow());
            for (script, problems) in &self.scripts {
                println!("  - {}", format!("{script}:").red().bold());
                for problem in problems {
                    println!("    - {}", problem.red());
                }
            }

            println!("{}", "💡 Tip: you can use `uvenv reinstall <package>` to reinstall an environment, which might fix the missing scripts.".blue());
        }

        issue_count
    }
}

impl CheckOptions {
    const fn to_metadataconfig(&self) -> LoadMetadataConfig {
        LoadMetadataConfig {
            recheck_scripts: !self.skip_scripts,
            updates_check: !self.skip_updates,
            updates_prereleases: self.show_prereleases,
            updates_ignore_constraints: self.ignore_constraints,
        }
    }
}

impl Process for CheckOptions {
    async fn process(self) -> anyhow::Result<i32> {
        let config = self.to_metadataconfig();

        let items = list_packages(&config, Some(&self.venv_names), None).await?;

        let mut issues = Issues::new();

        for metadata in &items {
            let invalid_scripts = metadata.invalid_scripts();
            if !self.skip_scripts && !invalid_scripts.is_empty() {
                issues.scripts.insert(&metadata.name, invalid_scripts);
            }

            if !self.skip_updates && metadata.outdated {
                issues.outdated.push(&metadata.name);
            }
        }

        if self.json {
            issues.print_json()
        } else {
            Ok(issues.print_human())
        }
    }
}
