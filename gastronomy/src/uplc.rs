use std::{
    collections::{BTreeMap, HashMap},
    ffi::OsStr,
    fs,
    path::{Path, PathBuf},
};

use anyhow::{Context, Result, anyhow};
use minicbor::bytes::ByteVec;
use pallas::ledger::{
    addresses::ScriptHash,
    primitives::conway::{self, Language, MintedTx},
};
use serde::Deserialize;
pub use uplc::ast::Program;
use uplc::{
    Fragment, PlutusData,
    ast::{DeBruijn, FakeNamedDeBruijn, Name, NamedDeBruijn},
    machine::{
        Machine, MachineState,
        cost_model::{CostModel, ExBudget},
        indexed_term::IndexedTerm,
    },
    parser,
    tx::{script_context::PlutusScript, tx_to_programs},
};

use crate::chain_query::ChainQuery;

pub struct LoadedProgram {
    pub filename: String,
    pub program: Program<NamedDeBruijn>,
    pub source_map: BTreeMap<u64, String>,
}

pub struct ScriptOverride {
    pub from_hash: ScriptHash,
    pub to_script: PlutusScript,
}

impl ScriptOverride {
    pub fn parse_key(key: String) -> Result<Self> {
        let parts: Vec<&str> = key.split(":").collect();

        if parts.len() != 3 {
            return Err(anyhow!("invalid script-override key. Expected hash:file"));
        }

        let from_hash = ScriptHash::from(hex::decode(parts[0])?.as_slice());

        let file_path = PathBuf::from(parts[1]);
        let version: usize = parts[2].to_string().parse()?;
        let path: &Path = file_path.as_ref();
        let absolute_path = if path.is_absolute() {
            path.to_path_buf()
        } else {
            std::env::current_dir()?.join(path)
        };

        Self::try_new(from_hash, absolute_path, version)
    }

    pub fn try_new(from_hash: ScriptHash, file_path: PathBuf, version: usize) -> Result<Self> {
        let bytes = fs::read(&file_path)?;
        let to_script = match version {
            1 => PlutusScript::V1(minicbor::decode::<conway::PlutusScript<1>>(
                bytes.as_slice(),
            )?),
            2 => PlutusScript::V2(minicbor::decode::<conway::PlutusScript<2>>(
                bytes.as_slice(),
            )?),
            3 => PlutusScript::V3(minicbor::decode::<conway::PlutusScript<3>>(
                bytes.as_slice(),
            )?),
            _ => {
                return Err(anyhow!(
                    "invalid version value. Only 1,2, and 3 are supported"
                ));
            }
        };

        Ok(Self {
            from_hash,
            to_script,
        })
    }
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

fn fix_names(program: Program<NamedDeBruijn>) -> Result<Program<NamedDeBruijn>> {
    let debruijn: Program<DeBruijn> = program.into();
    let name: Program<Name> = debruijn.try_into()?;
    let named_de_bruijn: Program<NamedDeBruijn> = name.try_into()?;
    Ok(named_de_bruijn)
}

fn load_flat(bytes: &[u8]) -> Result<Program<NamedDeBruijn>> {
    let fake_named_de_bruijn: Program<FakeNamedDeBruijn> = Program::from_flat(bytes)?;
    Ok(fake_named_de_bruijn.into())
}

pub async fn load_programs_from_file(
    file: &Path,
    query: ChainQuery,
    script_overrides: HashMap<ScriptHash, PlutusScript>,
) -> Result<Vec<LoadedProgram>> {
    let filename = file.display().to_string();
    match identify_file_type(file)? {
        FileType::Uplc => {
            let code = fs::read_to_string(file)?;
            let program = parser::program(&code).unwrap().try_into()?;
            let source_map = BTreeMap::new();
            Ok(vec![LoadedProgram {
                filename,
                program,
                source_map,
            }])
        }
        FileType::Flat => {
            let bytes = std::fs::read(file)?;
            let program = fix_names(load_flat(&bytes)?)?;
            let source_map = BTreeMap::new();
            Ok(vec![LoadedProgram {
                filename,
                program,
                source_map,
            }])
        }
        FileType::Json => {
            let export: AikenExport = serde_json::from_slice(&fs::read(file)?)?;
            let bytes = hex::decode(&export.compiled_code)?;
            let cbor: ByteVec = minicbor::decode(&bytes)?;
            let program = fix_names(load_flat(&cbor)?)?;
            let source_map = export.source_map.unwrap_or_default();
            Ok(vec![LoadedProgram {
                filename,
                program,
                source_map,
            }])
        }
        FileType::TransactionId => {
            let tx_id = hex::decode(file.to_str().unwrap())?;
            let tx_bytes = query.get_tx_bytes(tx_id[..].into()).await?;
            let multi_era_tx = MintedTx::decode_fragment(&tx_bytes).unwrap();
            load_programs_from_tx(filename, multi_era_tx, query, script_overrides).await
        }
        FileType::Transaction => {
            let bytes = std::fs::read(file)?;
            let multi_era_tx = MintedTx::decode_fragment(&bytes).unwrap();
            load_programs_from_tx(filename, multi_era_tx, query, script_overrides).await
        }
    }
}

async fn load_programs_from_tx(
    filename: String,
    tx: MintedTx<'_>,
    query: ChainQuery,
    script_overrides: HashMap<ScriptHash, PlutusScript>,
) -> Result<Vec<LoadedProgram>> {
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

    let mut programs = vec![];
    for (_, program, _) in
        tx_to_programs(&tx, &resolved_inputs, &slot_config, script_overrides).unwrap()
    {
        programs.push(LoadedProgram {
            filename: filename.clone(),
            program: fix_names(program)?,
            source_map: BTreeMap::new(),
        });
    }
    Ok(programs)
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
        filename,
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
        filename,
        program,
        source_map,
    })
}

pub fn execute_program(program: Program<NamedDeBruijn>) -> Result<Vec<(MachineState, ExBudget)>> {
    let mut machine = Machine::new(
        program.plutus_version()?,
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

/**
UTILITY LOGIC
*/
pub trait HasPlutusVersion {
    fn plutus_version(&self) -> Result<Language>;
}

impl<T> HasPlutusVersion for Program<T> {
    fn plutus_version(&self) -> Result<Language> {
        match self.version.0 {
            1 => Ok(Language::PlutusV1),
            2 => Ok(Language::PlutusV2),
            3 => Ok(Language::PlutusV3),
            _ => Err(anyhow!("invalid language version")),
        }
    }
}
