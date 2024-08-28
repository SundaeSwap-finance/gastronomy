use std::path::PathBuf;

use anyhow::Result;
use app::App;
use clap::{command, Parser, Subcommand};

mod app;
mod utils;

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
            let raw_program = gastronomy::uplc::parse_program(&file)?;
            let arguments = parameters
                .iter()
                .enumerate()
                .map(|(index, param)| gastronomy::uplc::parse_parameter(index, param.clone()))
                .collect::<Result<Vec<_>>>()?;
            let applied_program = gastronomy::uplc::apply_parameters(raw_program, arguments)?;
            let states = gastronomy::uplc::execute_program(applied_program)?;

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
