use std::{collections::HashMap, fs::remove_dir_all};

use clap::Parser;
use owo_colors::OwoColorize;
use proc_exit::Code;
use tracing::{info, instrument};
use xshell::{Shell, cmd};

use crate::{
    config::read_config,
    error::{Result, exit},
    repositories::{Repositories, build},
};

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

        let repositories = build(
            &self.repositories,
            config.exclude.as_deref().unwrap_or_default(),
        );

        let mut check_results = CheckResults::default();

        let mut failed = false;

        for repository in repositories {
            let name = repository.name();

            info!("{}", name.blue());

            check_results.current = Some(name.to_string());
            repository.check(&mut check_results, &config)?;

            if !check_results.checks_per_repo[name]
                .iter()
                .all(|check| check_results.checked[check].0.is_none())
            {
                for check in &check_results.checks_per_repo[name] {
                    let check_result = &check_results.checked[check];

                    if let Some(msg) = &check_result.0 {
                        let status = if check_result.1 {
                            "warn".yellow().to_string()
                        } else {
                            failed = true;
                            "fail".red().to_string()
                        };

                        info!("  {} {} - {msg}", status, check.cyan());
                    } else {
                        info!("  {} {}", "pass".green(), check.cyan());
                    }
                }
            }
        }

        if failed {
            exit(Code::FAILURE);
        }

        Ok(())
    }
}

#[derive(Debug, Default)]
pub struct CheckResults {
    current: Option<String>,
    checked: HashMap<String, (Option<String>, bool)>,
    checks_per_repo: HashMap<String, Vec<String>>,
}

impl CheckResults {
    pub fn has_checked(&mut self, name: &str) -> bool {
        let checked = self.checked.contains_key(name);

        if checked {
            self.add_check_to_repo(name);
        }

        checked
    }

    fn add_check_to_repo(&mut self, name: impl Into<String>) {
        self.checks_per_repo
            .entry(self.current.clone().unwrap())
            .or_default()
            .push(name.into());
    }

    #[inline]
    pub fn add_result(&mut self, name: &str, result: Option<impl Into<String>>) {
        self.add_result_warn(name, result, false);
    }

    pub fn add_result_warn(&mut self, name: &str, result: Option<impl Into<String>>, warn: bool) {
        self.checked
            .insert(name.to_string(), (result.map(|s| s.into()), warn));
        self.add_check_to_repo(name);
    }
}

pub fn check_program(
    sh: &Shell,
    results: &mut CheckResults,
    program: &str,
    command: &str,
    expect: &str,
) {
    if !results.has_checked(program) {
        let output = cmd!(sh, "sh -c {command}").quiet().ignore_status().read();
        let error = format!("{program} is not installed");

        results.add_result(
            program,
            if let Ok(output) = output {
                (!output.contains(expect)).then_some(error)
            } else {
                Some(error)
            },
        );
    }
}

pub fn check_git(sh: &Shell, results: &mut CheckResults) {
    check_program(sh, results, "git", "git --version", "git version");
}

pub fn check_repo(
    sh: &Shell,
    remote: &str,
    branch: &str,
    results: &mut CheckResults,
    warn: bool,
) -> Result {
    if results
        .checked
        .get("git")
        .is_some_and(|(check, _)| check.is_some())
    {
        results.add_result("repo", Some("git is not installed"));
        return Ok(());
    }

    if cmd!(sh, "git ls-remote --exit-code {remote}")
        .quiet()
        .ignore_stdout()
        .read_stderr()
        .is_err()
    {
        results.add_result_warn("repo", Some("repository not found or is empty"), warn);
        return Ok(());
    }

    if cmd!(sh, "git ls-remote --exit-code --heads {remote} {branch}")
        .quiet()
        .read()
        .is_err()
    {
        results.add_result("repo", Some("repository branch 'master' does not exist"));
        return Ok(());
    }

    let dir = "/tmp/publisher-check";

    if cmd!(sh, "git clone {remote} {dir}")
        .quiet()
        .read_stderr()
        .is_err()
    {
        results.add_result("repo", Some("read access to the repository not configured"));
        return Ok(());
    }

    sh.change_dir(dir);

    let push_result = cmd!(sh, "git push").quiet().read_stderr();

    sh.change_dir("..");
    remove_dir_all(dir)?;

    if push_result.is_err() {
        results.add_result(
            "repo",
            Some("write access to the repository not configured"),
        );
    }

    Ok(())
}
