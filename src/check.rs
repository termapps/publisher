use std::{collections::HashMap, io::Write};

use crate::{
    error::Result,
    repositories::{build_config, Repositories},
};

use anstream::{print, println, stdout};
use clap::Parser;
use owo_colors::OwoColorize;
use tracing::instrument;
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
        let repositories = build_config(&self.repositories);
        let mut check_results = CheckResults::default();

        for repository in repositories {
            let name = repository.name();

            print!("{} {} ... ", "checking".magenta(), name.yellow());
            stdout().flush()?;

            check_results.current = Some(name);
            repository.check(&mut check_results)?;

            if check_results.checked.values().all(Option::is_none) {
                println!("{}", "pass".green());
            } else {
                println!("{}", "fail".red());

                for check in &check_results.checks_per_repo[name] {
                    let check_result = check_results.checked[check];

                    if check_result.is_some() {
                        print!("  {}", "fail".red());
                    } else {
                        print!("  {}", "pass".green());
                    }

                    println!(" {}", check.yellow());

                    if let Some(msg) = check_result {
                        println!("    {}", msg);
                    }
                }
            }
        }

        Ok(())
    }
}

#[derive(Debug, Default)]
pub struct CheckResults {
    current: Option<&'static str>,
    checked: HashMap<&'static str, Option<&'static str>>,
    checks_per_repo: HashMap<&'static str, Vec<&'static str>>,
}

impl CheckResults {
    #[instrument(skip(self))]
    pub fn has_checked(&mut self, name: &'static str) -> bool {
        let checked = self.checked.contains_key(name);

        if checked {
            self.add_check_to_repo(name);
        }

        checked
    }

    #[instrument(skip(self))]
    fn add_check_to_repo(&mut self, name: &'static str) {
        self.checks_per_repo
            .entry(self.current.unwrap())
            .or_default()
            .push(name);
    }

    #[instrument(skip(self))]
    pub fn add_result(&mut self, name: &'static str, result: Option<&'static str>) {
        self.checked.insert(name, result);
        self.add_check_to_repo(name);
    }
}

#[instrument(skip_all)]
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

#[instrument(skip_all)]
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
