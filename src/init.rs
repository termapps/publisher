use std::{error::Error as StdError, fs::write, path::Path, result::Result as StdResult};

use clap::{Parser, ValueEnum};
use inquire::{
    required,
    validator::{ErrorMessage, Validation},
    MultiSelect, Text,
};
use regex::Regex;
use toml::to_string;
use tracing::instrument;

use crate::{
    config::{read_cargo_config, AppConfig, CONFIG_FILE},
    error::Result,
    repositories::{
        aur::AurConfig, aur_bin::AurBinConfig, homebrew::HomebrewConfig, nix::NixConfig,
        scoop::ScoopConfig, Repositories,
    },
};

type ValidatorResult = StdResult<Validation, Box<(dyn StdError + Send + Sync + 'static)>>;

/// Setup configuration for publishing to package repositories
#[derive(Debug, Parser)]
pub struct Init {}

impl Init {
    #[instrument(name = "init", skip_all)]
    pub fn run(self) -> Result {
        let package = read_cargo_config()?;

        let name = Text::new("Name of the binary?")
            .with_initial_value(&package.name.clone().unwrap_or_default())
            .with_validator(required!())
            .prompt()?;

        let description = Text::new("Description?")
            .with_initial_value(&package.description.clone().unwrap_or_default())
            .with_validator(required!())
            .prompt()?;

        let homepage = Text::new("Homepage?")
            .with_initial_value(&package.homepage.clone().unwrap_or_default())
            .with_validator(required!())
            .prompt()?;

        let license = Text::new("SPDX identifier of the license?")
            .with_initial_value(&package.license.clone().unwrap_or_default())
            .with_placeholder("MIT")
            .with_validator(required!())
            .prompt()?;

        let repository = Text::new("GitHub repository URI?")
            .with_placeholder("termapps/publisher")
            .with_validator(required!())
            .with_validator(repo_uri_validator)
            .prompt()?;

        let package_repositories = MultiSelect::new(
            "Package repositories to publish to?",
            Repositories::value_variants().into(),
        )
        .with_all_selected_by_default()
        .prompt()?;

        let homebrew = if package_repositories
            .iter()
            .any(|r| *r == Repositories::Homebrew)
        {
            let homebrew_name = Text::new("Homebrew formula name?")
                .with_initial_value(&name)
                .with_validator(required!())
                .prompt()?;

            let homebrew_repository = Text::new("Homebrew tap GitHub repository URI?")
                .with_placeholder("termapps/homebrew-tap")
                .with_validator(required!())
                .with_validator(repo_uri_validator)
                .prompt()?;

            Some(HomebrewConfig {
                name: (homebrew_name != name).then_some(homebrew_name),
                repository: homebrew_repository,
            })
        } else {
            None
        };

        let aur = if package_repositories.iter().any(|r| *r == Repositories::Aur) {
            let aur_name = Text::new("AUR package name?")
                .with_initial_value(&name)
                .with_validator(required!())
                .prompt()?;

            let different_name = aur_name != name;

            different_name.then_some(AurConfig {
                name: Some(aur_name),
                conflicts: None,
            })
        } else {
            None
        };

        let aur_bin = if package_repositories
            .iter()
            .any(|r| *r == Repositories::AurBin)
        {
            let package_name = format!("{name}-bin");

            let aur_bin_name = Text::new("AUR (bin) package name?")
                .with_initial_value(&package_name)
                .with_validator(required!())
                .prompt()?;

            let different_name = aur_bin_name != package_name;

            different_name.then_some(AurBinConfig {
                name: Some(aur_bin_name),
                conflicts: None,
            })
        } else {
            None
        };

        let scoop = if package_repositories
            .iter()
            .any(|r| *r == Repositories::Scoop)
        {
            let scoop_name = Text::new("Scoop app name?")
                .with_initial_value(&name)
                .with_validator(required!())
                .prompt()?;

            let scoop_repository = Text::new("Scoop bucket GitHub repository URI?")
                .with_placeholder("termapps/scoop-bucket")
                .with_validator(required!())
                .with_validator(repo_uri_validator)
                .prompt()?;

            Some(ScoopConfig {
                name: (scoop_name != name).then_some(scoop_name),
                repository: scoop_repository,
            })
        } else {
            None
        };

        let nix = if package_repositories.iter().any(|r| *r == Repositories::Nix) {
            let nix_name = Text::new("Nix package name?")
                .with_initial_value(&name)
                .with_validator(required!())
                .prompt()?;

            let nix_repository = Text::new("Nix package GitHub repository URI?")
                .with_initial_value(&repository)
                .with_validator(required!())
                .with_validator(repo_uri_validator)
                .prompt()?;

            let different_name = nix_name != name;
            let different_repo = nix_repository != repository;

            (different_name || different_repo).then_some(NixConfig {
                name: different_name.then_some(nix_name),
                repository: different_repo.then_some(nix_repository),
                path: None,
                lockfile: None,
            })
        } else {
            None
        };

        let exclude = Repositories::value_variants()
            .iter()
            .filter(|r| !package_repositories.contains(r))
            .map(|r| r.to_possible_value().unwrap().get_name().into())
            .collect::<Vec<_>>();

        let config = AppConfig {
            name: if name == package.name.unwrap_or_default() {
                "".into()
            } else {
                name
            },
            description: if description == package.description.unwrap_or_default() {
                "".into()
            } else {
                description
            },
            homepage: if homepage == package.homepage.unwrap_or_default() {
                "".into()
            } else {
                homepage
            },
            license: if license == package.license.unwrap_or_default() {
                "".into()
            } else {
                license
            },
            repository,
            exclude: (!exclude.is_empty()).then_some(exclude),
            aur,
            aur_bin,
            homebrew,
            scoop,
            nix,
        };

        write(Path::new(CONFIG_FILE), to_string(&config)?)?;

        Ok(())
    }
}

fn repo_uri_validator(val: &str) -> ValidatorResult {
    let pattern = Regex::new(r"^[a-zA-Z0-9_-]+/[a-zA-Z0-9_-]+$")?;

    if !pattern.is_match(val) {
        return Ok(Validation::Invalid(ErrorMessage::Custom(
            "Invalid URI format. Allowed format is '[name]/[name]'".to_string(),
        )));
    }

    Ok(Validation::Valid)
}
