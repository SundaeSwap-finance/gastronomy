use figment::providers::{Format, Toml};
pub use figment::Figment;
use serde::Deserialize;

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Config {
    pub blockfrost: Option<BlockfrostConfig>,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BlockfrostConfig {
    pub key: String,
}

pub fn load_base_config() -> Figment {
    let mut figment = Figment::new().merge(Toml::file(".gastronomyrc.toml"));
    if let Some(home_dir) = home::home_dir() {
        figment = figment.merge(Toml::file(home_dir.join(".gastronomyrc.toml")));
    }
    figment
}
