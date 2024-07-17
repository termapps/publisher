use config::{Config, File, FileFormat};

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

pub fn read_config() -> Result<AppConfig> {
    Ok(Config::builder()
        .add_source(File::new(CONFIG_FILE, FileFormat::Toml))
        .build()?
        .try_deserialize::<AppConfig>()?)
}
