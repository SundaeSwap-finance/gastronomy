use std::{ffi::OsStr, fs, path::Path};

use anyhow::{anyhow, Context, Result};
use pallas::ledger::primitives::conway::{Language, MintedTx, TransactionOutput};
use uplc::{
    ast::{FakeNamedDeBruijn, NamedDeBruijn, Program},
    machine::{
        cost_model::{CostModel, ExBudget},
        Machine, MachineState,
    },
    parser,
    tx::{tx_to_programs, ResolvedInput, SlotConfig},
    Fragment, PlutusData, TransactionInput,
};

pub enum FileType {
    UPLC,
    Flat,
    Transaction,
}

pub fn identify_file_type(file: &Path) -> Result<FileType> {
    let extension = file.extension().and_then(OsStr::to_str);
    match extension {
        Some("uplc") => Ok(FileType::UPLC),
        Some("flat") => Ok(FileType::Flat),
        Some("tx") => Ok(FileType::Transaction),
        _ => Err(anyhow!("That extension is not supported.")),
    }
}

pub fn load_programs_from_file(file: &Path) -> Result<Vec<Program<NamedDeBruijn>>> {
    match identify_file_type(file)? {
        FileType::UPLC => {
            let code = fs::read_to_string(file)?;
            let program = parser::program(&code).unwrap().try_into()?;
            Ok(vec![program])
        }
        FileType::Flat => {
            let bytes = std::fs::read(file)?;
            let program = Program::<FakeNamedDeBruijn>::from_flat(&bytes)?.into();
            Ok(vec![program])
        }
        FileType::Transaction => {
            let bytes = std::fs::read(file)?;
            let multi_era_tx = MintedTx::decode_fragment(&bytes).unwrap();
            load_programs_from_tx(multi_era_tx)
        }
    }
}

pub fn load_programs_from_tx(tx: MintedTx) -> Result<Vec<Program<NamedDeBruijn>>> {
    // TODO: actually look this stuff up
    let raw_inputs = hex::decode("84825820b16778c9cf065d9efeefe37ec269b4fc5107ecdbd0dd6bf3274b224165c2edd9008258206c732139de33e916342707de2aebef2252c781640326ff37b86ec99d97f1ba8d01825820975c17a4fed0051be622328efa548e206657d2b65a19224bf6ff8132571e6a500282582018f86700660fc88d0370a8f95ea58f75507e6b27a18a17925ad3b1777eb0d77600").unwrap();
    let raw_outputs = hex::decode("8482581d60b6c8794e9a7a26599440a4d0fd79cd07644d15917ff13694f1f67235821a000f8548a1581c15be994a64bdb79dde7fe080d8e7ff81b33a9e4860e9ee0d857a8e85a144576177610182581d60b6c8794e9a7a26599440a4d0fd79cd07644d15917ff13694f1f672351b00000001af14b8b482581d60b6c8794e9a7a26599440a4d0fd79cd07644d15917ff13694f1f672351a0098968082581d60b6c8794e9a7a26599440a4d0fd79cd07644d15917ff13694f1f672351a00acd8c6").unwrap();

    let inputs = Vec::<TransactionInput>::decode_fragment(&raw_inputs).unwrap();
    let outputs = Vec::<TransactionOutput>::decode_fragment(&raw_outputs).unwrap();
    let utxos: Vec<ResolvedInput> = inputs
        .iter()
        .zip(outputs.iter())
        .map(|(input, output)| ResolvedInput {
            input: input.clone(),
            output: output.clone(),
        })
        .collect();

    let slot_config = SlotConfig {
        zero_time: 1660003200000, // Preview network
        zero_slot: 0,
        slot_length: 1000,
    };

    Ok(tx_to_programs(&tx, &utxos, &slot_config)
        .unwrap()
        .drain(..)
        .map(|p| p.1)
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
    program: Program<NamedDeBruijn>,
    parameters: Vec<PlutusData>,
) -> Result<Program<NamedDeBruijn>> {
    let mut program = program;
    for (_, param) in parameters.iter().enumerate() {
        program = program.apply_data(param.clone());
    }
    Ok(program)
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
        state = machine
            .step(state)
            .map_err(|err| anyhow!("could not evaluate state: {}", err))?;
        states.push((state.clone(), machine.ex_budget));
    }

    Ok(states)
}
