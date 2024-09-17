use std::{collections::BTreeMap, ffi::OsStr, fs, path::Path};

use anyhow::{anyhow, Context, Result};
use minicbor::bytes::ByteVec;
use pallas::ledger::primitives::conway::{Language, MintedTx};
use serde::Deserialize;
use uplc::{
    ast::{FakeNamedDeBruijn, NamedDeBruijn, Program},
    machine::{
        cost_model::{CostModel, ExBudget},
        indexed_term::IndexedTerm,
        Machine, MachineState,
    },
    parser,
    tx::tx_to_programs,
    Fragment, PlutusData,
};

use crate::chain_query::ChainQuery;

pub struct LoadedProgram {
    pub program: Program<NamedDeBruijn>,
    pub source_map: BTreeMap<u64, String>,
}

enum FileType {
    Uplc,
    Flat,
    Json,
    Transaction,
    TransactionId,
}

fn identify_file_type(file: &Path) -> Result<FileType> {
    if let Some(path) = file.to_str() {
        if path.len() == 64 && hex::decode(path).is_ok() {
            return Ok(FileType::TransactionId);
        }
    }
    let extension = file.extension().and_then(OsStr::to_str);
    match extension {
        Some("uplc") => Ok(FileType::Uplc),
        Some("flat") => Ok(FileType::Flat),
        Some("json") => Ok(FileType::Json),
        Some("tx") => Ok(FileType::Transaction),
        _ => Err(anyhow!("That extension is not supported.")),
    }
}

pub async fn load_programs_from_file(file: &Path, query: ChainQuery) -> Result<Vec<LoadedProgram>> {
    match identify_file_type(file)? {
        FileType::Uplc => {
            let code = fs::read_to_string(file)?;
            let program = parser::program(&code).unwrap().try_into()?;
            let source_map = BTreeMap::new();
            Ok(vec![LoadedProgram {
                program,
                source_map,
            }])
        }
        FileType::Flat => {
            let bytes = std::fs::read(file)?;
            let program = Program::<FakeNamedDeBruijn>::from_flat(&bytes)?.into();
            let source_map = BTreeMap::new();
            Ok(vec![LoadedProgram {
                program,
                source_map,
            }])
        }
        FileType::Json => {
            let export: AikenExport = serde_json::from_slice(&fs::read(file)?)?;
            let bytes = hex::decode(&export.compiled_code)?;
            let cbor: ByteVec = minicbor::decode(&bytes)?;
            let program = Program::<FakeNamedDeBruijn>::from_flat(&cbor)?.into();
            let source_map = export.source_map.unwrap_or_default();
            Ok(vec![LoadedProgram {
                program,
                source_map,
            }])
        }
        FileType::TransactionId => {
            let tx_id = hex::decode(file.to_str().unwrap())?;
            let tx_bytes = query.get_tx_bytes(tx_id[..].into()).await?;
            let multi_era_tx = MintedTx::decode_fragment(&tx_bytes).unwrap();
            load_programs_from_tx(multi_era_tx, query).await
        }
        FileType::Transaction => {
            let bytes = std::fs::read(file)?;
            let multi_era_tx = MintedTx::decode_fragment(&bytes).unwrap();
            load_programs_from_tx(multi_era_tx, query).await
        }
    }
}

async fn load_programs_from_tx(tx: MintedTx<'_>, query: ChainQuery) -> Result<Vec<LoadedProgram>> {
    println!("loading programs from tx");
    let mut inputs: Vec<_> = tx.transaction_body.inputs.iter().cloned().collect();
    if let Some(ref_inputs) = &tx.transaction_body.reference_inputs {
        for input in ref_inputs.iter() {
            inputs.push(input.clone());
        }
    }
    if let Some(collateral) = &tx.transaction_body.collateral {
        for input in collateral.iter() {
            inputs.push(input.clone());
        }
    }
    let resolved_inputs = query.get_utxos(inputs).await?;
    let slot_config = query.get_slot_config()?;
    println!("resolved inputs");

    Ok(tx_to_programs(&tx, &resolved_inputs, &slot_config)
        .unwrap()
        .drain(..)
        .map(|p| LoadedProgram {
            program: p.1,
            source_map: BTreeMap::new(),
        })
        .collect())
}

pub fn parse_parameter(index: usize, parameter: String) -> Result<PlutusData> {
    let data: PlutusData = {
        let bytes =
            hex::decode(parameter).context(format!("could not hex-decode parameter {}", index))?;
        uplc::plutus_data(&bytes).map_err(|e| {
            anyhow!(
                "could not decode plutus data for parameter {}: {}",
                index,
                e
            )
        })?
    };
    Ok(data)
}

pub fn apply_parameters(
    LoadedProgram {
        program,
        source_map,
    }: LoadedProgram,
    parameters: Vec<PlutusData>,
) -> Result<LoadedProgram> {
    let mut program = program;
    let mut source_map_offset = 0;
    for param in parameters {
        program = program.apply_data(param);
        source_map_offset += 1;
    }
    // Every time we apply a parameter, it adds another root term wrapping the program.
    // That messes with offsets from our source maps, so make sure to update those.
    let source_map = source_map
        .into_iter()
        .map(|(index, location)| (index + source_map_offset, location))
        .collect();
    Ok(LoadedProgram {
        program,
        source_map,
    })
}

pub fn execute_program(program: Program<NamedDeBruijn>) -> Result<Vec<(MachineState, ExBudget)>> {
    let mut machine = Machine::new(
        Language::PlutusV2,
        CostModel::default(),
        ExBudget::default(),
        1,
    );
    let mut state = machine
        .get_initial_machine_state(program.term)
        .map_err(|err| anyhow!("could not get initial state: {}", err))?;
    let mut states = vec![(state.clone(), machine.ex_budget)];
    while !matches!(state, MachineState::Done(_)) {
        state = match machine.step(state) {
            Ok(state) => state,
            Err(err) => {
                eprintln!("Machine Error: {}", err);
                MachineState::Done(IndexedTerm::Error { index: None })
            }
        };
        states.push((state.clone(), machine.ex_budget));
    }

    Ok(states)
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct AikenExport {
    compiled_code: String,
    source_map: Option<BTreeMap<u64, String>>,
}
