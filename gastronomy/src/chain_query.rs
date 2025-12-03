use anyhow::{Result, bail};
use blockfrost::{BlockfrostAPI, JsonValue};
use blockfrost_openapi::models::script::Type;
use pallas::{
    applying::utils::{AlonzoError, ValidationError, add_values},
    codec::utils::{Bytes, CborWrap, NonEmptyKeyValuePairs, PositiveCoin},
    ledger::{
        addresses::Address,
        primitives::conway::{
            self, DatumOption, PostAlonzoTransactionOutput, PseudoScript, TransactionOutput,
        },
    },
};
use serde_json::from_str;
use uplc::{
    Fragment, Hash, KeyValuePairs, PlutusData, TransactionInput, Value,
    tx::{ResolvedInput, SlotConfig},
};

use crate::config::BlockfrostConfig;

pub enum ChainQuery {
    Blockfrost(Blockfrost),
    None,
}

impl ChainQuery {
    pub fn blockfrost(config: &BlockfrostConfig) -> Self {
        Self::Blockfrost(Blockfrost::new(config))
    }

    pub async fn get_tx_bytes(&self, tx_id: Hash<32>) -> Result<Bytes> {
        match self {
            Self::Blockfrost(blockfrost) => blockfrost.get_tx_bytes(tx_id).await,
            Self::None => {
                bail!("No chain query provider configured, consider adding a blockfrost API key")
            }
        }
    }

    pub async fn get_utxos(&self, tx_ref: Vec<TransactionInput>) -> Result<Vec<ResolvedInput>> {
        match self {
            Self::Blockfrost(blockfrost) => blockfrost.get_utxos(tx_ref).await,
            Self::None => {
                bail!("No chain query provider configured, consider adding a blockfrost API key")
            }
        }
    }

    pub fn get_slot_config(&self) -> Result<SlotConfig> {
        match self {
            Self::Blockfrost(blockfrost) => blockfrost.get_slot_config(),
            Self::None => {
                bail!("No chain query provider configured, consider adding a blockfrost API key")
            }
        }
    }
}

impl Default for ChainQuery {
    fn default() -> Self {
        Self::None
    }
}

trait ChainQueryImpl {
    async fn get_tx_bytes(&self, tx_id: Hash<32>) -> Result<Bytes>;
    async fn get_utxos(&self, tx_ref: Vec<TransactionInput>) -> Result<Vec<ResolvedInput>>;
    fn get_slot_config(&self) -> Result<SlotConfig>;
}

pub struct Blockfrost {
    api_key: String,
    environment: Environment,
    api: BlockfrostAPI,
}

enum Environment {
    Preview,
    Preprod,
    Mainnet,
}

impl Environment {
    fn as_str(&self) -> &'static str {
        match self {
            Self::Preview => "preview",
            Self::Preprod => "preprod",
            Self::Mainnet => "mainnet",
        }
    }
    fn slot_config(&self) -> SlotConfig {
        let (zero_time, zero_slot) = match self {
            Self::Preview => (1666656000000, 0),
            Self::Preprod => (1655683200000, 0),
            Self::Mainnet => (1591566291000, 0),
        };
        SlotConfig {
            slot_length: 1000,
            zero_slot,
            zero_time,
        }
    }
}

impl Blockfrost {
    pub fn new(config: &BlockfrostConfig) -> Self {
        let environment = if config.key.starts_with("preview") {
            Environment::Preview
        } else if config.key.starts_with("preprod") {
            Environment::Preprod
        } else {
            Environment::Mainnet
        };
        Blockfrost {
            api_key: config.key.to_string(),
            environment,
            api: BlockfrostAPI::new(&config.key, Default::default()),
        }
    }
}

impl ChainQueryImpl for Blockfrost {
    async fn get_tx_bytes(&self, tx_id: Hash<32>) -> Result<Bytes> {
        let client = reqwest::Client::new();
        let tx_id = hex::encode(tx_id);
        let response = client
            .get(format!(
                "https://cardano-{}.blockfrost.io/api/v0/txs/{}/cbor",
                self.environment.as_str(),
                tx_id
            ))
            .header("project_id", self.api_key.as_str())
            .send()
            .await?;
        let value = from_str::<JsonValue>(&response.text().await?)?;
        let tx_bytes = hex::decode(value["cbor"].as_str().unwrap())?;
        Ok(tx_bytes.into())
    }
    async fn get_utxos(&self, inputs: Vec<TransactionInput>) -> Result<Vec<ResolvedInput>> {
        let mut resolved_inputs = vec![];
        for input in inputs {
            let tx = self
                .api
                .transactions_utxos(hex::encode(input.transaction_id).as_str())
                .await?;
            let output = tx.outputs[input.index as usize].clone();
            let datum = match (output.inline_datum, output.data_hash) {
                (Some(datum), _) => Some(DatumOption::Data(CborWrap(
                    hex::decode(datum)
                        .ok()
                        .and_then(|d| PlutusData::decode_fragment(&d).ok())
                        .unwrap(),
                ))),
                (None, Some(hash)) => {
                    Some(DatumOption::Hash(hex::decode(hash).unwrap()[..].into()))
                }
                (None, None) => None,
            };
            let mut value: Value = pallas::applying::utils::empty_value();
            for asset in output.amount.iter() {
                if asset.unit == "lovelace" {
                    value = add_values(
                        &value,
                        &Value::Coin(asset.quantity.parse().unwrap()),
                        &ValidationError::Alonzo(AlonzoError::NegativeValue),
                    )
                    .unwrap();
                } else {
                    let policy: Hash<28> = hex::decode(&asset.unit[0..56]).unwrap()[..].into();
                    let asset_name: Bytes = hex::decode(&asset.unit[56..]).unwrap().into();
                    let amount: u64 = asset.quantity.parse().unwrap();
                    let asset_amt = KeyValuePairs::Def(vec![(asset_name, amount)]);
                    let multiasset = KeyValuePairs::Def(vec![(policy, asset_amt)]);
                    value = add_values(
                        &value,
                        &Value::Multiasset(0u64, multiasset),
                        &ValidationError::Alonzo(AlonzoError::NegativeValue),
                    )
                    .unwrap();
                }
            }
            let value = match value {
                Value::Coin(coin) => conway::Value::Coin(coin),
                Value::Multiasset(coin, multiasset) => conway::Value::Multiasset(
                    coin,
                    NonEmptyKeyValuePairs::Def(
                        multiasset
                            .iter()
                            .map(|(k, v)| {
                                (
                                    *k,
                                    NonEmptyKeyValuePairs::Def(
                                        v.iter()
                                            .map(|(k, v)| {
                                                (k.clone(), PositiveCoin::try_from(*v).unwrap())
                                            })
                                            .collect(),
                                    ),
                                )
                            })
                            .collect(),
                    ),
                ),
            };

            let script_ref = if let Some(script_hash) = output.reference_script_hash {
                let script_version = self.api.scripts_by_id(script_hash.as_str()).await?.r#type;
                let script = self
                    .api
                    .scripts_datum_hash_cbor(script_hash.as_str())
                    .await?;
                let bytes = hex::decode(script["cbor"].as_str().unwrap()).unwrap();
                let script = match script_version {
                    Type::PlutusV1 => {
                        PseudoScript::PlutusV1Script(conway::PlutusScript(bytes.into()))
                    }
                    Type::PlutusV2 => {
                        PseudoScript::PlutusV2Script(conway::PlutusScript(bytes.into()))
                    }
                    Type::PlutusV3 => {
                        PseudoScript::PlutusV3Script(conway::PlutusScript(bytes.into()))
                    }
                    Type::Timelock => {
                        unreachable!("native script cannot be a used referenced in a script_ref")
                    }
                };

                Some(CborWrap(script))
            } else {
                None
            };

            let output: TransactionOutput =
                TransactionOutput::PostAlonzo(PostAlonzoTransactionOutput {
                    address: Address::from_bech32(&output.address)?.to_vec().into(),
                    datum_option: datum,
                    script_ref,
                    value,
                });
            resolved_inputs.push(ResolvedInput { input, output });
        }
        Ok(resolved_inputs)
    }
    fn get_slot_config(&self) -> Result<SlotConfig> {
        Ok(self.environment.slot_config())
    }
}
