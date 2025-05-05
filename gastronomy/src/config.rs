use std::path::PathBuf;

pub use figment::Figment;
use figment::providers::{Format, Toml};
use pallas::ledger::addresses::ScriptHash;
use serde::Deserialize;

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Config {
    pub blockfrost: Option<BlockfrostConfig>,
    pub blueprint_path: Option<PathBuf>,
    pub script_overrides: Option<Vec<ScriptOverrideConfig>>,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BlockfrostConfig {
    pub key: String,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ScriptOverrideConfig {
    pub from: String,
    pub to: String,
}

pub type ScriptOverride = (ScriptHash, ScriptHash);

impl TryFrom<ScriptOverrideConfig> for ScriptOverride {
    type Error = anyhow::Error;

    fn try_from(value: ScriptOverrideConfig) -> Result<Self, Self::Error> {
        let from_hash = ScriptHash::from(hex::decode(value.from)?.as_slice());
        let to_hash = ScriptHash::from(hex::decode(value.to)?.as_slice());

        Ok((from_hash, to_hash))
    }
}

pub fn load_base_config() -> Figment {
    let mut figment = Figment::new().merge(Toml::file(".gastronomyrc.toml"));
    if let Some(home_dir) = home::home_dir() {
        figment = figment.merge(Toml::file(home_dir.join(".gastronomyrc.toml")));
    }
    figment
}
