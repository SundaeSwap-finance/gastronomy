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
use uplc::parser::term;
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
                        let name = Rc::make_mut(name);
                        let text = terms_to_readable_names
                            .entry(name.unique)
                            .or_insert(name.text.clone() + "-" + &name.unique.to_string())
                            .to_string();
                        *term = Term::Var(
                            Name {
                                text,
                                unique: name.unique,
                            }
                            .into(),
                        )
                    }
                    Term::Delay(name) => {
                        println!("Delay name - {}", name);
                    }
                    Term::Lambda {
                        parameter_name,
                        body,
                    } => {
                        let name = Rc::make_mut(parameter_name);
                        let text = terms_to_readable_names
                            .entry(name.unique)
                            .or_insert(name.text.clone() + "-" + &name.unique.to_string())
                            .to_string();
                        *term = Term::Lambda {
                            parameter_name: Name::text(text).into(),
                            body: Rc::new(body.as_ref().clone()),
                        }
                    }
                    Term::Apply { function, argument } => {
                        println!("Apply function - {}", function);
                        println!("Apply argument - {}", argument);
                    }
                    Term::Constant(name) => {
                        println!("Constant name - {:?}", name);
                    }
                    Term::Force(name) => {
                        println!("Force name - {:?}", name);
                    }
                    Term::Error => {
                        println!("Error");
                    }
                    Term::Builtin(name) => {
                        println!("Builtin name - {}", name);
                    }
                    Term::Constr { tag, fields } => {
                        println!("Contr tag - {}", tag);
                        println!("Contr fields - {:?}", fields);
                    }
                    Term::Case { constr, branches } => {
                        println!("Case constr - {}", constr);
                        println!("Case branches - {:?}", branches);
                    }
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
