use std::{ffi::OsStr, fs, path::Path};

use anyhow::{anyhow, Context, Result};
use pallas::ledger::primitives::babbage::Language;
use uplc::{
    ast::{FakeNamedDeBruijn, NamedDeBruijn, Program},
    machine::{
        cost_model::{CostModel, ExBudget},
        Machine, MachineState,
    },
    parser, PlutusData,
};

pub fn parse_program(file: &Path) -> Result<Program<NamedDeBruijn>> {
    let program: Program<NamedDeBruijn> =
        if file.extension().and_then(OsStr::to_str) == Some("uplc") {
            let code = fs::read_to_string(file)?;
            parser::program(&code).unwrap().try_into()?
        } else if file.extension().and_then(OsStr::to_str) == Some("flat") {
            let bytes = std::fs::read(file)?;
            Program::<FakeNamedDeBruijn>::from_flat(&bytes)?.into()
        } else {
            return Err(anyhow!("That extension is not supported."));
        };
    Ok(program)
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
