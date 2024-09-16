use std::{
    collections::HashMap,
    fmt::{Debug, Display, Formatter, Result as FmtResult},
};

pub mod aur;
pub mod aur_bin;
pub mod homebrew;
pub mod nix;
pub mod scoop;

use clap::ValueEnum;
use xshell::{cmd, Shell};

use crate::{check::CheckResults, config::AppConfig, error::Result, targets::Target};

pub trait Repository {
    fn name(&self) -> &'static str;

    fn check(&self, check_result: &mut CheckResults, info: &AppConfig) -> Result;

    fn publish(&self, info: &AppConfig, version: &str, dry_run: bool) -> Result;

    fn instructions(&self, info: &AppConfig) -> Result<Vec<String>>;
}

#[derive(Debug, Clone, PartialEq, Eq, ValueEnum)]
pub enum Repositories {
    Aur,
    AurBin,
    Homebrew,
    Scoop,
    Nix,
}

impl Display for Repositories {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(f, "{}", self.build().name())
    }
}

impl Repositories {
    pub fn build(&self) -> Box<dyn Repository> {
        match self {
            Repositories::Aur => Box::new(aur::Aur),
            Repositories::AurBin => Box::new(aur_bin::AurBin),
            Repositories::Homebrew => Box::new(homebrew::Homebrew),
            Repositories::Scoop => Box::new(scoop::Scoop),
            Repositories::Nix => Box::new(nix::Nix),
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

pub fn update_config(repositories: &[Repositories], exclude: &[String], config: &mut AppConfig) {
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

fn get_checksums(
    info: &AppConfig,
    version: &str,
    targets: Vec<Target>,
) -> Result<HashMap<Target, String>> {
    let AppConfig {
        name, repository, ..
    } = info;

    let sh = Shell::new()?;
    let download_url =
        format!("https://github.com/{repository}/releases/download/v{version}/{name}-v{version}");

    targets
        .into_iter()
        .map(|target| {
            let target_str = if target != Target::Source {
                format!("-{target}")
            } else {
                format!("")
            };

            let checksum = cmd!(sh, "curl -L {download_url}{target_str}_sha256sum.txt")
                .ignore_stderr()
                .read()?;

            Ok((target, checksum))
        })
        .collect()
}
