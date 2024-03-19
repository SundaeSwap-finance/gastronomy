mod app;
mod utils;
mod nuplc;

use std::path::PathBuf;

use app::App;
use clap::{command, Parser, Subcommand};
use pallas::ledger::primitives::babbage::Language;
use uplc::ast::{FakeNamedDeBruijn, NamedDeBruijn, Program};
use uplc::machine::cost_model::{CostModel, ExBudget};
use uplc::machine::{Machine, MachineState};

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand, Debug)]
enum Commands {
    Run { file: PathBuf },
}

fn main() -> Result<(), anyhow::Error> {
    utils::install_hooks().unwrap();

    let args = Args::parse();

    match args.command {
        Some(Commands::Run { file }) => {
            let bytes = std::fs::read(file.clone())?;
            let program: Program<NamedDeBruijn> =
                Program::<FakeNamedDeBruijn>::from_flat(&bytes)?.into();
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
            utils::restore()?;
            Ok(app_result.unwrap()) // TODO
        }
        None => {
            println!("No command provided");
            Ok(())
        }
    }
}
