use config::{Config, File, FileFormat};
use serde::Deserialize;
use serde_json::from_str;
use xshell::{cmd, Shell};

use crate::{
    error::Result,
    repositories::{
        aur::AurConfig, aur_bin::AurBinConfig, homebrew::HomebrewConfig, scoop::ScoopConfig,
    },
};

pub const CONFIG_FILE: &str = "publisher.toml";

#[derive(Debug, Clone, serde::Deserialize)]
pub struct AppConfig {
    pub name: String,
    pub description: String,
    pub homepage: String,
    pub license: String,
    pub repository: String,
    pub exclude: Option<Vec<String>>,
    pub aur: Option<AurConfig>,
    pub aur_bin: Option<AurBinConfig>,
    pub homebrew: Option<HomebrewConfig>,
    pub scoop: Option<ScoopConfig>,
}

#[derive(Debug, Default, Deserialize)]
struct CargoMetadataPackage {
    pub name: Option<String>,
    pub description: Option<String>,
    pub homepage: Option<String>,
    pub license: Option<String>,
}

#[derive(Debug, Default, Deserialize)]
struct CargoMetadata {
    pub packages: Vec<CargoMetadataPackage>,
}

pub fn read_config() -> Result<AppConfig> {
    let sh = Shell::new()?;

    // Read cargo metadata if exists
    let metadata = cmd!(sh, "cargo metadata --no-deps --format-version 1")
        .quiet()
        .read()
        .ok()
        .unwrap_or_else(|| "{}".into());

    let metadata = from_str::<CargoMetadata>(&metadata).unwrap_or_default();
    let mut builder = Config::builder();

    if !metadata.packages.is_empty() {
        let package = &metadata.packages[0];

        builder = builder
            .set_default("name", package.name.clone())?
            .set_default("description", package.description.clone())?
            .set_default("homepage", package.homepage.clone())?
            .set_default("license", package.license.clone())?;
    }

    Ok(builder
        .add_source(File::new(CONFIG_FILE, FileFormat::Toml))
        .build()?
        .try_deserialize::<AppConfig>()?)
}
