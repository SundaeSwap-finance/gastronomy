mod app;
mod utils;

use std::collections::HashMap;
use std::ffi::OsStr;
use std::path::PathBuf;
use std::rc::Rc;
use std::{fs, process};

use app::App;
use clap::{command, Parser, Subcommand};
use pallas::codec::minicbor::decode::Error;
use pallas::ledger::primitives::babbage::Language;

use uplc::ast::{FakeNamedDeBruijn, Name, NamedDeBruijn, Program, Term, Unique};
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
                    let bytes = hex::decode(param)
                        .map_err::<Error, _>(|e| {
                            Error::message(format!("Invalid hex-encoded string: {e}")).into()
                        })
                        .unwrap_or_else(|e| {
                            println!("{}", e);
                            process::exit(1)
                        });

                    uplc::plutus_data(&bytes)
                        .map_err::<Error, _>(|e| {
                            Error::message(format!(
                                "Invalid Plutus data; malformed CBOR encoding: {e}"
                            ))
                            .into()
                        })
                        .unwrap_or_else(|e| {
                            println!("{}", e);
                            process::exit(1)
                        })
                };
                program = program.apply_data(data);
            }

            let program: Program<Name> = Program::<Name>::try_from(program)?.try_into()?;

            let mut terms_to_readable_names: HashMap<Unique, String> = HashMap::new();

            let program = program
                .clone()
                .traverse_uplc_with(&mut |_, term, _, _| match term {
                    Term::Var(name) => {
                        *term = update_terms(name, &mut terms_to_readable_names);
                        println!("{:?}", terms_to_readable_names);
                    }
                    Term::Delay(name) => println!("Delay - {}", term),
                    Term::Lambda {
                        parameter_name,
                        body,
                    } => {
                        println!("paramter_name - {:?}", parameter_name);
                        println!("body - {:?}", body);
                        *term = update_terms(parameter_name, &mut terms_to_readable_names);
                        println!("{:?}", terms_to_readable_names);
                    }
                    Term::Apply { function, argument } => println!("Apply - {}", term),
                    Term::Constant(name) => println!("Constant - {}", term),
                    Term::Force(name) => println!("Force - {}", term),
                    Term::Error => println!("Error"),
                    Term::Builtin(name) => println!("Builtin - {}", term),
                    Term::Constr { tag, fields } => println!("Contr - {}", term),
                    Term::Case { constr, branches } => println!("Case - {}", term),
                });

            let program: Program<NamedDeBruijn> =
                Program::<NamedDeBruijn>::try_from(program)?.try_into()?;

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

fn update_terms(
    name: &mut Rc<Name>,
    terms_to_readable_names: &mut HashMap<Unique, String>,
) -> Term<Name> {
    let name = Rc::make_mut(name);
    let text = terms_to_readable_names
        .entry(name.unique)
        .or_insert(name.text.clone() + "-" + &name.unique.to_string())
        .to_string();
    Term::Var(
        Name {
            text: text,
            unique: name.unique,
        }
        .into(),
    )
}
