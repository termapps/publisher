mod aur;

use clap::ValueEnum;

use crate::{check::CheckResults, error::Result, publish::PublishInfo};

pub trait Repository {
    fn name(&self) -> &'static str;

    fn check(&self, check_result: &mut CheckResults) -> Result;

    fn publish(&self, info: &PublishInfo, version: &str) -> Result;
}

#[derive(Debug, Clone, ValueEnum)]
pub enum Repositories {
    Aur,
}

impl Repositories {
    pub fn build_config(&self) -> impl Repository {
        match self {
            Repositories::Aur => aur::Aur,
        }
    }
}

pub fn build_config(repositories: &[Repositories]) -> Vec<impl Repository> {
    if !repositories.is_empty() {
        repositories.iter()
    } else {
        Repositories::value_variants().iter()
    }
    .map(Repositories::build_config)
    .collect()
}
