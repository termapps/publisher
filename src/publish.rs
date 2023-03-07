use crate::{
    error::Result,
    repositories::{build_config, Repositories, Repository},
};

use clap::Parser;
use config::{Config, Environment, File, FileFormat};

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
            repository.publish(&config, &self.version)?;
        }

        Ok(())
    }
}
