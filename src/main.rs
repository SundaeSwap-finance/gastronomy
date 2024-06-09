mod app;
mod utils;

use std::ffi::OsStr;
use std::fs;
use std::path::PathBuf;

use anyhow::{anyhow, Context};
use app::App;
use clap::{command, Parser, Subcommand};

use pallas::ledger::primitives::babbage::Language;
use uplc::ast::{FakeNamedDeBruijn, NamedDeBruijn, Program};
use uplc::machine::cost_model::{CostModel, ExBudget};
use uplc::machine::{Machine, MachineState};
use uplc::{parser, PlutusData};

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand, Debug)]
enum Commands {
    Run {
        file: PathBuf,
        parameters: Vec<String>,
    },
}

fn main() -> Result<(), anyhow::Error> {
    utils::install_hooks().unwrap();

    let args = Args::parse();

    match args.command {
        Some(Commands::Run { file, parameters }) => {
            let mut program: Program<NamedDeBruijn> =
                if file.extension().and_then(OsStr::to_str) == Some("uplc") {
                    let code = fs::read_to_string(file.clone())?;
                    parser::program(&code).unwrap().try_into()?
                } else if file.extension().and_then(OsStr::to_str) == Some("flat") {
                    let bytes = std::fs::read(file.clone())?;
                    Program::<FakeNamedDeBruijn>::from_flat(&bytes)?.into()
                } else {
                    println!("That file extension is not supported.");
                    return Ok(());
                };

            for param in parameters {
                let data: PlutusData = {
                    let bytes = hex::decode(param).context("could not hex-decode parameter")?;

                    uplc::plutus_data(&bytes)
                        .map_err(|e| anyhow!("could not decode plutus data: {}", e))?
                };
                program = program.apply_data(data);
            }

            let mut machine = Machine::new(
                Language::PlutusV2,
                CostModel::default(),
                ExBudget::default(),
                1,
            );
            let mut state = machine.get_initial_machine_state(program.term).unwrap();
            let mut states = vec![(state.clone(), machine.ex_budget)];
            loop {
                state = machine.step(state).unwrap();
                states.push((state.clone(), machine.ex_budget));
                if let MachineState::Done(_) = state {
                    break;
                }
            }
            let mut terminal = utils::init()?;
            let mut app = App {
                file_name: file,
                cursor: 0,
                states,
                exit: false,
                focus: "Term".into(),
                ..Default::default()
            };
            let app_result = app.run(&mut terminal);
            utils::restore().and(app_result)?;
            Ok(())
        }
        None => {
            println!("No command provided");
            Ok(())
        }
    }
}
