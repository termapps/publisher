use std::{fmt::Debug, fs::write};

use crate::{
    error::Result,
    repositories::{build_config, homebrew::HomebrewInfo, Repositories, Repository},
};

use clap::Parser;
use config::{Config, Environment, File, FileFormat};
use owo_colors::OwoColorize;
use tracing::{info, instrument};
use xshell::{cmd, Shell};

/// Publish the tool to package repositories
#[derive(Debug, Parser)]
pub struct Publish {
    /// Version to publish
    version: String,

    /// The name(s) of the package repository
    repositories: Vec<Repositories>,
}

#[derive(Debug, Clone, serde::Deserialize)]
pub struct PublishInfo {
    pub name: String,
    pub description: String,
    pub repository: String,
    pub license: String,
    pub homepage: String,
    pub homebrew: Option<HomebrewInfo>,
}

impl Publish {
    #[instrument(name = "publish", skip_all)]
    pub fn run(self) -> Result {
        let repositories = build_config(&self.repositories);

        let config = read_config()?;

        for repository in repositories {
            info!("{}", repository.name().yellow());
            repository.publish(&config, &self.version)?;
        }

        Ok(())
    }
}

pub fn read_config() -> Result<PublishInfo> {
    Ok(Config::builder()
        .add_source(Environment::with_prefix("PUBLISHER").separator("_"))
        .add_source(File::new("publisher.toml", FileFormat::Toml))
        .build()?
        .try_deserialize::<PublishInfo>()?)
}

pub fn prepare_tmp_dir(repository: &dyn Repository) -> Result<(Shell, String)> {
    let sh = Shell::new()?;

    let name = repository.name();
    let dir = format!("/tmp/publisher/{name}");

    cmd!(sh, "rm -rf {dir}").quiet().run()?;
    cmd!(sh, "mkdir -p {dir}").quiet().run()?;

    sh.change_dir(&dir);

    Ok((sh, dir))
}

pub fn prepare_git_repo(repository: &dyn Repository, remote: &str) -> Result<(Shell, String)> {
    let (sh, dir) = prepare_tmp_dir(repository)?;

    let name = repository.name();

    cmd!(sh, "git init").quiet().ignore_stdout().run()?;
    cmd!(sh, "git remote add {name} {remote}")
        .quiet()
        .ignore_stdout()
        .run()?;
    cmd!(sh, "git fetch {name}").quiet().ignore_stderr().run()?;

    if let Ok(_) = cmd!(sh, "git ls-remote --exit-code --heads {name} master")
        .quiet()
        .ignore_stdout()
        .run()
    {
        cmd!(sh, "git checkout master")
            .quiet()
            .ignore_stdout()
            .ignore_stderr()
            .run()?;
    }

    Ok((sh, dir))
}

pub fn write_and_add<P, F>(sh: &Shell, dir: &str, path: P, writer: F) -> Result
where
    P: AsRef<str> + Debug,
    F: FnOnce() -> Vec<String>,
{
    let path = path.as_ref();

    info!("  {} {}", "writing".magenta(), path.yellow());
    let lines = writer();

    write(format!("{dir}/{path}"), lines.join("\n"))?;
    cmd!(sh, "git add {path}").quiet().run()?;

    Ok(())
}

// TODO: Add name to this
pub fn commit_and_push(
    repository: &dyn Repository,
    sh: &Shell,
    name: &str,
    version: &str,
) -> Result {
    let remote = repository.name();
    let message = format!("{name}: {version}");

    cmd!(sh, "git commit -m {message}")
        .quiet()
        .ignore_stdout()
        .run()?;
    cmd!(sh, "git push {remote} master")
        .quiet()
        .ignore_stderr()
        .run()?;

    Ok(())
}
