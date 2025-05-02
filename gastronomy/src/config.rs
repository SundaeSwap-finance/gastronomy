use std::path::{Path, PathBuf};

pub use figment::Figment;
use figment::providers::{Format, Toml};
use pallas::ledger::primitives::ScriptHash;
use serde::Deserialize;

use crate::uplc::ScriptOverride;

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Config {
    pub blockfrost: Option<BlockfrostConfig>,
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
    pub file_path: String,
    pub from_hash: String,
    pub script_version: usize,
}

impl TryFrom<ScriptOverrideConfig> for ScriptOverride {
    type Error = anyhow::Error;

    fn try_from(value: ScriptOverrideConfig) -> Result<Self, Self::Error> {
        let from_hash = ScriptHash::from(hex::decode(value.from_hash)?.as_slice());

        let file_path = PathBuf::from(value.file_path);
        let path: &Path = file_path.as_ref();
        let absolute_path = if path.is_absolute() {
            path.to_path_buf()
        } else {
            std::env::current_dir()?.join(path)
        };

        ScriptOverride::try_new(from_hash, absolute_path, value.script_version)
    }
}

pub fn load_base_config() -> Figment {
    let mut figment = Figment::new().merge(Toml::file(".gastronomyrc.toml"));
    if let Some(home_dir) = home::home_dir() {
        figment = figment.merge(Toml::file(home_dir.join(".gastronomyrc.toml")));
    }
    figment
}
