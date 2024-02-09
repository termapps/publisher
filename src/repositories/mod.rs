use std::fmt::Debug;

pub mod aur;
pub mod homebrew;

use clap::ValueEnum;

use crate::{check::CheckResults, error::Result, publish::PublishInfo};

pub trait Repository {
    fn name(&self) -> &'static str;

    fn check(&self, check_result: &mut CheckResults, info: &PublishInfo) -> Result;

    fn publish(&self, info: &PublishInfo, version: &str) -> Result;
}

#[derive(Debug, Clone, ValueEnum)]
pub enum Repositories {
    Aur,
    Homebrew,
}

impl Repositories {
    fn build_config(&self) -> Box<dyn Repository> {
        match self {
            Repositories::Aur => Box::new(aur::Aur),
            Repositories::Homebrew => Box::new(homebrew::Homebrew),
        }
    }
}

pub fn build_config(repositories: &[Repositories]) -> Vec<Box<dyn Repository>> {
    if !repositories.is_empty() {
        repositories.iter()
    } else {
        Repositories::value_variants().iter()
    }
    .map(Repositories::build_config)
    .collect()
}
