use owo_colors::OwoColorize;
use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::cli::{CheckOptions, Process};
use crate::commands::list::list_packages;
use crate::helpers::ResultToString;
use crate::metadata::LoadMetadataConfig;

#[derive(Debug, PartialEq, Deserialize, Serialize)]
struct Issues {
    outdated: Vec<String>,
    scripts: BTreeMap<String, Vec<String>>,
}

impl Issues {
    pub fn new() -> Self {
        Issues {
            outdated: Vec::new(),
            scripts: BTreeMap::new(),
        }
    }

    pub fn count_outdated(&self) -> i32 {
        self.outdated.len() as i32
    }
    pub fn count_scripts(&self) -> i32 {
        self.scripts
            .values()
            .fold(0, |acc, vec| acc + vec.len() as i32)
    }

    pub fn count(&self) -> i32 {
        self.count_outdated() + self.count_scripts()
    }
}

impl CheckOptions {
    async fn process_human(
        self,
        issues: &Issues,
    ) -> Result<i32, String> {
        let issue_count = issues.count();

        if issue_count == 0 {
            println!("{}", "✅ No issues found. Everything is up-to-date and all scripts are properly installed!".green().bold());
            return Ok(0);
        }

        println!("{}", "🚨 Issues Overview:".bold().underline());

        // Display outdated issues
        if !issues.outdated.is_empty() {
            println!("{}", "\n🔶 Outdated:".bold().yellow());
            for issue in &issues.outdated {
                println!("  - {}", issue.red());
            }

            println!("{}", "💡 Tip: you can use `uvx upgrade <package>` to update all outdated dependencies in a specific environment.".blue());
        }

        // Display script issues
        if !issues.scripts.is_empty() {
            println!("{}", "\n🔶 Missing Scripts:".bold().yellow());
            for (script, problems) in &issues.scripts {
                println!("  - {}", format!("{}:", script).red().bold());
                for problem in problems {
                    println!("    - {}", problem.red());
                }
            }

            println!("{}", "💡 Tip: you can use `uvx reinstall <package>` to reinstall an environment, which might fix the missing scripts.".blue());
        }

        Ok(issue_count)
    }

    async fn process_json(
        self,
        issues: &Issues,
    ) -> Result<i32, String> {
        let json = serde_json::to_string_pretty(issues).map_err_to_string()?;

        eprintln!("{}", json);

        Ok(issues.count())
    }

    fn to_metadataconfig(&self) -> LoadMetadataConfig {
        LoadMetadataConfig {
            recheck_scripts: !self.skip_scripts,
            updates_check: !self.skip_updates,
            updates_prereleases: self.show_prereleases,
            updates_ignore_constraints: self.ignore_constraints,
        }
    }
}

impl Process for CheckOptions {
    async fn process(self) -> Result<i32, String> {
        let config = self.to_metadataconfig();

        let mut items = list_packages(&config).await?;

        if !self.venv_names.is_empty() {
            items.retain(|k| self.venv_names.contains(&k.name))
        }

        let mut issues = Issues::new();

        for metadata in items {
            let invalid_scripts = metadata.invalid_scripts();
            if !self.skip_scripts && !invalid_scripts.is_empty() {
                issues
                    .scripts
                    .insert(metadata.name.clone(), invalid_scripts);
            }

            if !self.skip_updates && metadata.outdated {
                issues.outdated.push(metadata.name.clone())
            }
        }

        if self.json {
            self.process_json(&issues).await
        } else {
            self.process_human(&issues).await
        }
    }
}
