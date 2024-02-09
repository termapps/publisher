use crate::{
    error::Result,
    repositories::{build_config, Repositories, Repository},
};

use clap::Parser;
use config::{Config, Environment, File, FileFormat};
use tracing::info;
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
}

impl Publish {
    pub fn run(self) -> Result {
        let repositories = build_config(&self.repositories);

        let config = Config::builder()
            .add_source(Environment::with_prefix("PUBLISHER").separator("_"))
            .add_source(File::new("publisher.toml", FileFormat::Toml))
            .build()?
            .try_deserialize::<PublishInfo>()?;

        for repository in repositories {
            info!("Publishing to {}", repository.name());
            repository.publish(&config, &self.version)?;
        }

        Ok(())
    }
}

pub fn prepare_tmp_dir(repository: &dyn Repository) -> Result<(Shell, String)> {
    let sh = Shell::new()?;

    let name = repository.name();
    let dir = format!("/tmp/publisher/{name}");

    cmd!(sh, "rm -rf {dir}").run()?;
    cmd!(sh, "mkdir -p {dir}").run()?;

    sh.change_dir(&dir);

    Ok((sh, dir))
}

pub fn prepare_git_repo(repository: &dyn Repository, remote: &str) -> Result<(Shell, String)> {
    let (sh, dir) = prepare_tmp_dir(repository)?;

    let name = repository.name();

    cmd!(sh, "git init").run()?;
    cmd!(sh, "git remote add {name} {remote}").run()?;
    cmd!(sh, "git fetch {name}").run()?;

    if let Ok(_) = cmd!(sh, "git ls-remote --exit-code --heads {name} master").run() {
        cmd!(sh, "git checkout master").run()?;
    }

    Ok((sh, dir))
}

pub fn commit_and_push(repository: &dyn Repository, sh: &Shell, version: &str) -> Result {
    let name = repository.name();

    cmd!(sh, "git commit -m {version}").run()?;
    cmd!(sh, "git push {name} master").run()?;

    Ok(())
}
