use std::{
    collections::HashMap,
    io::{self, Write},
};

use crate::{
    error::Result,
    repositories::{build_config, Repositories},
};

use clap::Parser;
use xshell::{cmd, Shell};

/// Check requirements for publishing to package repositories
#[derive(Debug, Parser)]
pub struct Check {
    /// The name(s) of the package repository
    repositories: Vec<Repositories>,
}

impl Check {
    pub fn run(self) -> Result {
        let repositories = build_config(&self.repositories);
        let mut check_results = CheckResults::default();

        for repository in repositories {
            let name = repository.name();

            print!("Checking {name}... ");
            io::stdout().flush()?;

            check_results.current = Some(name);
            repository.check(&mut check_results)?;

            if check_results.checked.values().all(Option::is_none) {
                println!("ok");
            } else {
                println!("failed");

                for check in &check_results.checks_per_repo[name] {
                    let check_result = check_results.checked[check];

                    println!(
                        "    {check} {}",
                        if check_result.is_some() {
                            "failed"
                        } else {
                            "ok"
                        }
                    );

                    if let Some(msg) = check_result {
                        println!("        {}", msg);
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
    #[inline]
    pub fn has_checked(&self, name: &'static str) -> bool {
        self.checked.contains_key(name)
    }

    #[inline]
    pub fn add_result(&mut self, name: &'static str, result: Option<&'static str>) {
        self.checked.insert(name, result);
        self.checks_per_repo
            .entry(self.current.unwrap())
            .or_default()
            .push(name);
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
