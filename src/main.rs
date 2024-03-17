use std::io;
use std::path::PathBuf;

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
    let args = Args::parse();

    match args.command {
        Some(Commands::Run { file }) => {
            let bytes = std::fs::read(file)?;
            let program: Program<NamedDeBruijn> =
                Program::<FakeNamedDeBruijn>::from_flat(&bytes)?.into();
            let mut machine = Machine::new(
                Language::PlutusV2,
                CostModel::default(),
                ExBudget::default(),
                200,
            );
            let mut state = machine.get_initial_machine_state(program.term).unwrap();
            let mut states = vec![(state.clone(), machine.ex_budget)];
            loop {
                state = machine.step(state).unwrap();
                states.push((state.clone(), machine.ex_budget));
                if let MachineState::Done(t) = state {
                    let spent_mem = 14000000 - machine.ex_budget.mem;
                    let spent_cpu = 10000000000 - machine.ex_budget.cpu;
                    println!("Finished. {} steps.", states.len());
                    println!(
                        "Budget -- Memory: {} ({:.3}%)   CPU: {} ({:.3}%)",
                        spent_mem,
                        100. * (spent_mem as f64) / 14000000.,
                        spent_cpu,
                        100. * (spent_cpu as f64) / 10000000000.
                    );
                    println!("Final value: {:?}", t);
                    break;
                }
            }
            let mut cursor = 0;
            loop {
                let mut buffer = String::new();
                let state = match &states[cursor].0 {
                    MachineState::Return(_, _) => "Return ",
                    MachineState::Compute(_, _, _) => "Compute",
                    MachineState::Done(_) => "Done   ",
                };
                let spent_mem = 14000000 - states[cursor].1.mem;
                let spent_cpu = 10000000000 - states[cursor].1.cpu;
                println!(
                    "Cursor: {:5} -- Next Instruction: {} -- Budget Spent: {} mem  {} cpu",
                    cursor, state, spent_mem, spent_cpu
                );
                println!("Commands: n (next), p (previous), s (state), c (context), e (env), v (value), q (quit)");
                io::stdin().read_line(&mut buffer)?;
                match &*buffer {
                    "q\n" => {
                        break;
                    }
                    "s\n" => match &states[cursor].0 {
                        MachineState::Return(_, _) => println!("About to execute: Return"),
                        MachineState::Compute(_, _, _) => println!("About to execute: Compute"),
                        MachineState::Done(_) => println!("Machine is done."),
                    },
                    "c\n" => match &states[cursor].0 {
                        MachineState::Return(c, _) | MachineState::Compute(c, _, _) => {
                            println!("Context: {:?}", c)
                        }
                        MachineState::Done(_) => println!("Machine is done, no context."),
                    },
                    "e\n" => match &states[cursor].0 {
                        MachineState::Compute(_, e, _) => println!("Env: {:?}", e),
                        MachineState::Return(_, _) => {
                            println!("Machine is returning a value, no Env.")
                        }
                        MachineState::Done(_) => println!("Machine is done, no context."),
                    },
                    "v\n" => match &states[cursor].0 {
                        MachineState::Compute(_, _, v) => println!("Value: {:?}", v),
                        MachineState::Return(_, v) => println!("Value: {:?}", v),
                        MachineState::Done(t) => println!("Machine finishd with term: {:?}.", t),
                    },
                    "\n" | "n\n" => {
                        let executed_state = match &states[cursor].0 {
                            MachineState::Return(_, _) => "Return",
                            MachineState::Compute(_, _, _) => "Compute",
                            MachineState::Done(_) => "Done",
                        };

                        println!("Executing: {}", executed_state);
                        cursor += 1;
                        if cursor >= states.len() {
                            cursor = states.len() - 1;
                            println!("Done.");
                        }
                    }
                    "p\n" => {
                        println!("Rewinding.");
                        if cursor == 0 {
                            println!("At beginning.");
                        } else {
                            cursor -= 1;
                        }
                    }
                    _ => {
                        println!("Unrecognized command.");
                    }
                }
            }
        }
        None => {
            println!("No command provided");
        }
    }
    Ok(())
}
