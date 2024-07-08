use std::fmt::Debug;

pub mod aur;
pub mod aur_bin;
pub mod homebrew;

use clap::ValueEnum;

use crate::{check::CheckResults, error::Result, publish::PublishInfo};

pub trait Repository {
    fn name(&self) -> &'static str;

    fn check(&self, check_result: &mut CheckResults, info: &PublishInfo) -> Result;

    fn publish(&self, info: &PublishInfo, version: &str, dry_run: bool) -> Result;
}

#[derive(Debug, Clone, PartialEq, Eq, ValueEnum)]
pub enum Repositories {
    AurBin,
    Aur,
    Homebrew,
}

impl Repositories {
    fn build(&self) -> Box<dyn Repository> {
        match self {
            Repositories::AurBin => Box::new(aur_bin::AurBin),
            Repositories::Aur => Box::new(aur::Aur),
            Repositories::Homebrew => Box::new(homebrew::Homebrew),
        }
    }
}

fn get_repositories<'a>(
    repositories: &'a [Repositories],
    exclude: &'a [String],
) -> Vec<&'a Repositories> {
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
        .collect()
}

pub fn build(repositories: &[Repositories], exclude: &[String]) -> Vec<Box<dyn Repository>> {
    get_repositories(repositories, exclude)
        .into_iter()
        .map(Repositories::build)
        .collect()
}

pub fn update_conflicts(
    repositories: &[Repositories],
    exclude: &[String],
    config: &mut PublishInfo,
) {
    let repos = get_repositories(repositories, exclude);

    // Add conflicts between AUR and AUR (bin) if both are selected
    if repos.iter().any(|r| r == &&Repositories::Aur)
        && repos.iter().any(|r| r == &&Repositories::AurBin)
    {
        let aur_name = aur::get_name(config);
        let aur_bin_name = aur_bin::get_name(config);

        config
            .aur
            .get_or_insert_with(Default::default)
            .conflicts
            .get_or_insert_with(Default::default)
            .push(aur_bin_name);
        config
            .aur_bin
            .get_or_insert_with(Default::default)
            .conflicts
            .get_or_insert_with(Default::default)
            .push(aur_name);
    }
}
