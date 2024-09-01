use anyhow::Result;
use blockfrost::{BlockfrostAPI, JsonValue};
use pallas::{
    applying::utils::{add_values, AlonzoError, ValidationError},
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
    tx::{ResolvedInput, SlotConfig},
    Fragment, Hash, KeyValuePairs, PlutusData, TransactionInput, Value,
};

pub trait ChainQuery {
    async fn get_tx_bytes(&self, tx_id: Hash<32>) -> Result<Bytes>;
    async fn get_utxos(&self, tx_ref: Vec<TransactionInput>) -> Result<Vec<ResolvedInput>>;
    fn get_slot_config(&self) -> Result<SlotConfig>;
}

pub struct None {}
impl ChainQuery for None {
    async fn get_tx_bytes(&self, _tx_id: Hash<32>) -> Result<Bytes> {
        unimplemented!("No chain query provider conigured, consider adding a blockfrost API key")
    }
    async fn get_utxos(&self, _tx_refs: Vec<TransactionInput>) -> Result<Vec<ResolvedInput>> {
        unimplemented!("No chain query provider conigured, consider adding a blockfrost API key")
    }
    fn get_slot_config(&self) -> Result<SlotConfig> {
        unimplemented!("No chain query provider conigured, consider adding a blockfrost API key")
    }
}

pub struct Blockfrost {
    api_key: String,
    api: BlockfrostAPI,
}

impl Blockfrost {
    pub fn new(api_key: &str) -> Self {
        Blockfrost {
            api_key: api_key.to_string(),
            api: BlockfrostAPI::new(api_key, Default::default()),
        }
    }
}

impl ChainQuery for Blockfrost {
    async fn get_tx_bytes(&self, tx_id: Hash<32>) -> Result<Bytes> {
        let client = reqwest::Client::new();
        let tx_id = hex::encode(tx_id);
        let response = client
            .get(format!(
                "https://cardano-preview.blockfrost.io/api/v0/txs/{}/cbor",
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
            let datum = if let Some(datum) = output.inline_datum {
                Some(DatumOption::Data(CborWrap(
                    hex::decode(datum)
                        .ok()
                        .and_then(|d| PlutusData::decode_fragment(&d).ok())
                        .unwrap(),
                )))
            } else if let Some(hash) = output.data_hash {
                Some(DatumOption::Hash(hex::decode(hash).unwrap()[..].into()))
            } else {
                None
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
                    let policy: Hash<28> =
                        hex::decode(asset.unit[0..56].to_string()).unwrap()[..].into();
                    let asset_name: Bytes =
                        hex::decode(asset.unit[56..].to_string()).unwrap().into();
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
                                    k.clone(),
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
                let script = self
                    .api
                    .scripts_datum_hash_cbor(script_hash.as_str())
                    .await?;
                let bytes = hex::decode(script["cbor"].as_str().unwrap()).unwrap();
                Some(CborWrap(PseudoScript::PlutusV2Script(
                    conway::PlutusV2Script(bytes.into()),
                )))
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
        Ok(SlotConfig {
            zero_time: 1660003200000, // Preview network
            zero_slot: 0,
            slot_length: 1000,
        })
    }
}
