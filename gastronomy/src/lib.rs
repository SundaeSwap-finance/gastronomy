pub mod chain_query;
pub mod config;
pub mod execution_trace;
pub mod uplc;

use std::{collections::HashMap, fs::File, io::BufReader, path::PathBuf, str::FromStr};

use anyhow::{Context, Result, anyhow};

//REXPORTS
use ::uplc::tx::script_context::PlutusScript;
use aiken_project::blueprint::Blueprint;
pub use execution_trace::Frame;
pub use hex;
pub use pallas::ledger::primitives::ScriptHash;

pub fn parse_script_overrides(
    script_overrides: Vec<String>,
) -> Result<Vec<(ScriptHash, ScriptHash)>> {
    script_overrides
        .into_iter()
        .map(|key| {
            let parts: Vec<&str> = key.split(":").collect();

            if parts.len() != 2 {
                return Err(anyhow!("invalid script-override key. Expected hash:hash"));
            }

            let from_hash = ScriptHash::from(hex::decode(parts[0])?.as_slice());
            let to_hash = ScriptHash::from(hex::decode(parts[1])?.as_slice());
            Ok((from_hash, to_hash))
        })
        .collect::<Result<Vec<_>, _>>()
}

pub fn compute_script_overrides(
    script_overrides: Vec<(ScriptHash, ScriptHash)>,
    maybe_blueprint: Option<PathBuf>,
) -> Result<HashMap<ScriptHash, PlutusScript>> {
    let mut overrides: HashMap<ScriptHash, PlutusScript> = HashMap::new();

    if !script_overrides.is_empty() {
        let blueprint_path = maybe_blueprint
            .unwrap_or(PathBuf::from_str("plutus.json").expect("Failed to create default PathBuf"));
        let blueprint =
            File::open(blueprint_path).map_err(|e| anyhow!("failed to open blueprint: {}", e))?;

        let blueprint: Blueprint = serde_json::from_reader(BufReader::new(blueprint))
            .context("failed to parse blueprint")?;
        let blueprint_validators: HashMap<ScriptHash, PlutusScript> = blueprint.into();

        script_overrides
            .iter()
            .try_for_each::<_, Result<_>>(|(from_hash, to_hash)| {
                overrides.insert(
                    from_hash.clone(),
                    blueprint_validators
                        .get(&to_hash)
                        .ok_or(anyhow!("script override not found for hash: {}", to_hash))?
                        .clone(),
                );

                Ok(())
            })?;
    }

    Ok(overrides)
}
