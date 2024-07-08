use std::path::Path;

use crate::{
    error::{Error, Result},
    publish::read_config,
    repositories::{build, Repositories},
};

use clap::Parser;
use owo_colors::OwoColorize;
use tracing::{info, instrument};
use xshell::{cmd, Shell};

pub const CONFIG_FILE: &str = "publisher.toml";

/// Initialize publishing to package repositories
#[derive(Debug, Parser)]
pub struct Init {
    /// The name(s) of the package repository
    repositories: Vec<Repositories>,
}

impl Init {
    #[instrument(name = "init", skip_all)]
    pub fn run(self) -> Result {
        let repositories = build(&self.repositories, &[]);

        Ok(())
    }
}
