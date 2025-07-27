use config::{Config, File, FileFormat};
use eyre::eyre;
use serde::{Deserialize, Serialize};
use serde_json::from_str;
use xshell::{Shell, cmd};

use crate::{
    error::Result,
    repositories::{
        aur::AurConfig, aur_bin::AurBinConfig, debian::DebianConfig, homebrew::HomebrewConfig,
        nix::NixConfig, scoop::ScoopConfig,
    },
};

pub const CONFIG_FILE: &str = "publisher.toml";

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct AppConfig {
    #[serde(skip_serializing_if = "String::is_empty")]
    pub name: String,
    #[serde(skip_serializing_if = "String::is_empty")]
    pub description: String,
    #[serde(skip_serializing_if = "String::is_empty")]
    pub homepage: String,
    #[serde(skip_serializing_if = "String::is_empty")]
    pub license: String,
    pub repository: String,
    pub exclude: Option<Vec<String>>,
    pub cargo: Option<String>,
    pub homebrew: Option<HomebrewConfig>,
    pub debian: Option<DebianConfig>,
    pub aur: Option<AurConfig>,
    pub aur_bin: Option<AurBinConfig>,
    pub scoop: Option<ScoopConfig>,
    pub nix: Option<NixConfig>,
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct CargoMetadataPackage {
    pub name: Option<String>,
    pub description: Option<String>,
    pub homepage: Option<String>,
    pub license: Option<String>,
    pub publish: Option<Vec<String>>,
}

#[derive(Debug, Clone, Default, Deserialize)]
struct CargoMetadata {
    pub packages: Vec<CargoMetadataPackage>,
}

pub fn read_cargo_config() -> Result<CargoMetadataPackage> {
    let sh = Shell::new()?;

    // Read cargo metadata if exists
    let metadata = cmd!(sh, "cargo metadata --no-deps --format-version 1")
        .quiet()
        .ignore_stderr()
        .read()
        .ok()
        .unwrap_or_else(|| "{}".into());

    let metadata = from_str::<CargoMetadata>(&metadata).unwrap_or_default();

    Ok(metadata.packages.first().cloned().unwrap_or_default())
}

pub fn read_config() -> Result<AppConfig> {
    let package = read_cargo_config()?;

    let builder = Config::builder()
        .set_default("name", package.name.clone())?
        .set_default("description", package.description.clone())?
        .set_default("homepage", package.homepage.clone())?
        .set_default("license", package.license.clone())?
        .set_default(
            "cargo",
            package.publish.is_none().then(|| package.name.clone()),
        )?;

    Ok(builder
        .add_source(File::new(CONFIG_FILE, FileFormat::Toml))
        .build()
        .map_err(|e| eyre!("Unable to parse the configuration file: {e}"))?
        .try_deserialize::<AppConfig>()?)
}
