use std::{
    fmt::Debug,
    fs::{create_dir_all, remove_dir_all, remove_file, write},
    path::Path,
};

use clap::Parser;
use owo_colors::OwoColorize;
use tracing::{info, instrument, warn};
use xshell::{Shell, cmd};

use crate::{
    config::read_config,
    error::Result,
    repositories::{Repositories, Repository, build, update_config},
};

/// Publish the tool to package repositories
#[derive(Debug, Parser)]
pub struct Publish {
    /// Version to publish
    version: String,

    /// The name(s) of the package repository
    repositories: Vec<Repositories>,

    /// Disable dry run mode
    #[clap(long)]
    no_dry_run: bool,
}

impl Publish {
    #[instrument(name = "publish", skip_all)]
    pub fn run(self) -> Result {
        let mut config = read_config()?;
        let exclude = config.exclude.clone().unwrap_or_default();

        // We need to update config depending on what user has provided
        update_config(&self.repositories, &exclude, &mut config);

        let repositories = build(&self.repositories, &exclude);

        for repository in repositories {
            info!("{}", repository.name().blue());
            repository.publish(&config, &self.version, !self.no_dry_run)?;
        }

        if !self.no_dry_run {
            warn!(
                "{}",
                "Not publishing because dry run mode is enabled".yellow()
            );
        }

        Ok(())
    }
}

pub fn prepare_tmp_dir(repository: &dyn Repository) -> Result<(Shell, String)> {
    let id = repository.name();

    let sh = Shell::new()?;
    let dir = format!("/tmp/publisher/{id}");

    remove_dir_all(&dir)?;
    create_dir_all(&dir)?;

    sh.change_dir(&dir);

    Ok((sh, dir))
}

pub fn prepare_git_repo(repository: &dyn Repository, remote: &str) -> Result<(Shell, String)> {
    let (sh, dir) = prepare_tmp_dir(repository)?;

    cmd!(sh, "git init").quiet().ignore_stdout().run()?;
    cmd!(sh, "git remote add origin {remote}")
        .quiet()
        .ignore_stdout()
        .run()?;
    cmd!(sh, "git fetch origin").quiet().ignore_stderr().run()?;

    if cmd!(sh, "git ls-remote --exit-code --heads origin master")
        .quiet()
        .ignore_stdout()
        .run()
        .is_ok()
    {
        cmd!(sh, "git checkout master")
            .quiet()
            .ignore_stdout()
            .ignore_stderr()
            .run()?;
    }

    Ok((sh, dir))
}

pub fn write_file<P, F>(dir: &str, path: P, writer: F) -> Result
where
    P: AsRef<str> + Debug,
    F: FnOnce() -> Vec<String>,
{
    let path = path.as_ref();
    let full_path = Path::new(dir).join(path);

    // Ensure the parent directory exists, otherwise fails on linux
    if let Some(parent) = full_path.parent() {
        create_dir_all(parent)?;
    }

    info!("  {:>11} {}", "writing".magenta(), path.cyan());
    let lines = writer();

    write(full_path, format!("{}\n", lines.join("\n")))?;

    Ok(())
}

pub fn write_and_add<P, F>(sh: &Shell, dir: &str, path: P, writer: F) -> Result
where
    P: AsRef<str> + Debug,
    F: FnOnce() -> Vec<String>,
{
    let path = path.as_ref();

    write_file(dir, path, writer)?;

    cmd!(sh, "git add {path}").quiet().run()?;

    Ok(())
}

pub fn commit_and_push(sh: &Shell, name: &str, version: &str) -> Result {
    let message = format!("{name}: {version}");

    cmd!(sh, "git commit -m {message}")
        .quiet()
        .ignore_stdout()
        .run()?;
    cmd!(sh, "git push origin master")
        .quiet()
        .ignore_stderr()
        .run()?;

    Ok(())
}
