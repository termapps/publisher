use config::{Config, File, FileFormat};
use eyre::eyre;
use serde::{Deserialize, Serialize};
use serde_json::from_str;
use xshell::{Shell, cmd};

use crate::{
    error::Result,
    repositories::{
        aur::AurConfig, aur_bin::AurBinConfig, homebrew::HomebrewConfig, nix::NixConfig,
        npm::NPMConfig, scoop::ScoopConfig,
    },
};

pub(crate) const CONFIG_FILE: &str = "publisher.toml";

#[derive(Debug, Clone, Deserialize, Serialize)]
pub(crate) struct AppConfig {
    #[serde(skip_serializing_if = "String::is_empty")]
    pub(crate) name: String,
    #[serde(skip_serializing_if = "String::is_empty")]
    pub(crate) description: String,
    #[serde(skip_serializing_if = "String::is_empty")]
    pub(crate) homepage: String,
    #[serde(skip_serializing_if = "String::is_empty")]
    pub(crate) license: String,
    pub(crate) repository: String,
    pub(crate) exclude: Option<Vec<String>>,
    pub(crate) cargo: Option<String>,
    pub(crate) homebrew: Option<HomebrewConfig>,
    pub(crate) aur: Option<AurConfig>,
    pub(crate) aur_bin: Option<AurBinConfig>,
    pub(crate) scoop: Option<ScoopConfig>,
    pub(crate) nix: Option<NixConfig>,
    pub(crate) npm: Option<NPMConfig>,
}

#[derive(Debug, Clone, Default, Deserialize)]
pub(crate) struct CargoMetadataPackage {
    pub(crate) name: Option<String>,
    pub(crate) description: Option<String>,
    pub(crate) homepage: Option<String>,
    pub(crate) license: Option<String>,
    pub(crate) publish: Option<Vec<String>>,
}

#[derive(Debug, Clone, Default, Deserialize)]
struct CargoMetadata {
    pub(crate) packages: Vec<CargoMetadataPackage>,
}

pub(crate) fn read_cargo_config() -> Result<CargoMetadataPackage> {
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

pub(crate) fn read_config() -> Result<AppConfig> {
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
