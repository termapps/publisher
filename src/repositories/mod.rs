use std::fmt::Debug;

pub mod aur;
pub mod aur_bin;
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
    AurBin,
    Aur,
    Homebrew,
}

impl Repositories {
    fn build_config(&self) -> Box<dyn Repository> {
        match self {
            Repositories::AurBin => Box::new(aur_bin::AurBin),
            Repositories::Aur => Box::new(aur::Aur),
            Repositories::Homebrew => Box::new(homebrew::Homebrew),
        }
    }
}

pub fn build_config(repositories: &[Repositories], exclude: &[String]) -> Vec<Box<dyn Repository>> {
    let repos = if !repositories.is_empty() {
        repositories.iter()
    } else {
        Repositories::value_variants().iter()
    };

    repos
        .filter(|r| {
            let v = r.to_possible_value().unwrap();
            !exclude.iter().any(|e| v.matches(e, true))
        })
        .map(Repositories::build_config)
        .collect()
}
