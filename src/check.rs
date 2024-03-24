use std::collections::HashMap;

use crate::{
    error::{Error, Result},
    publish::read_config,
    repositories::{build_config, Repositories},
};

use clap::Parser;
use owo_colors::OwoColorize;
use tracing::{info, instrument};
use xshell::{cmd, Shell};

/// Check requirements for publishing to package repositories
#[derive(Debug, Parser)]
pub struct Check {
    /// The name(s) of the package repository
    repositories: Vec<Repositories>,
}

impl Check {
    #[instrument(name = "check", skip_all)]
    pub fn run(self) -> Result {
        let config = read_config()?;

        let repositories = build_config(
            &self.repositories,
            config.exclude.as_deref().unwrap_or_default(),
        );

        let mut check_results = CheckResults::default();

        let mut failed = false;

        for repository in repositories {
            let name = repository.name();

            info!("{}", name.yellow());

            check_results.current = Some(name);
            repository.check(&mut check_results, &config)?;

            if !check_results.checks_per_repo[name]
                .iter()
                .all(|&check| check_results.checked[check].is_none())
            {
                for check in &check_results.checks_per_repo[name] {
                    let check_result = check_results.checked[check];

                    if let Some(msg) = check_result {
                        failed = true;
                        info!("  {} {} - {msg}", "fail".red(), check.yellow());
                    } else {
                        info!("  {} {}", "pass".green(), check.yellow());
                    }
                }
            }
        }

        if failed {
            Err(Error::ChecksFailed)
        } else {
            Ok(())
        }
    }
}

#[derive(Debug, Default)]
pub struct CheckResults {
    current: Option<&'static str>,
    checked: HashMap<&'static str, Option<&'static str>>,
    checks_per_repo: HashMap<&'static str, Vec<&'static str>>,
}

impl CheckResults {
    pub fn has_checked(&mut self, name: &'static str) -> bool {
        let checked = self.checked.contains_key(name);

        if checked {
            self.add_check_to_repo(name);
        }

        checked
    }

    fn add_check_to_repo(&mut self, name: &'static str) {
        self.checks_per_repo
            .entry(self.current.unwrap())
            .or_default()
            .push(name);
    }

    pub fn add_result(&mut self, name: &'static str, result: Option<&'static str>) {
        self.checked.insert(name, result);
        self.add_check_to_repo(name);
    }
}

pub fn check_git(sh: &Shell, results: &mut CheckResults) {
    if !results.has_checked(&"git") {
        let output = cmd!(sh, "git --version").quiet().ignore_status().read();

        results.add_result(
            "git",
            if let Ok(output) = output {
                (!output.contains("git version")).then_some("git is not installed")
            } else {
                Some("git is not installed")
            },
        );
    }
}

pub fn check_curl(sh: &Shell, results: &mut CheckResults) {
    if !results.has_checked(&"curl") {
        let output = cmd!(sh, "curl --version").quiet().ignore_status().read();

        results.add_result(
            "curl",
            if let Ok(output) = output {
                (!output.contains("curl ")).then_some("curl is not installed")
            } else {
                Some("curl is not installed")
            },
        );
    }
}
