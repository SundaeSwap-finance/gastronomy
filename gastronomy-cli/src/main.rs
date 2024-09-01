use std::path::PathBuf;

use anyhow::Result;
use app::App;
use clap::{command, Parser, Subcommand};
use gastronomy::chain_query::{Blockfrost, None};

mod app;
mod utils;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[command(subcommand)]
    command: Option<Commands>,
    #[clap(env)]
    blockfrost: String,
}

#[derive(Subcommand, Debug)]
enum Commands {
    Run {
        file: PathBuf,
        parameters: Vec<String>,
        #[clap(long)]
        index: Option<usize>,
    },
}

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    utils::install_hooks().unwrap();

    let args = Args::parse();

    match args.command {
        Some(Commands::Run {
            file,
            parameters,
            index,
        }) => {
            let mut raw_programs = if args.blockfrost.is_empty() {
                gastronomy::uplc::load_programs_from_file(&file, None {}).await?
            } else {
                gastronomy::uplc::load_programs_from_file(
                    &file,
                    Blockfrost::new(args.blockfrost.as_str()),
                )
                .await?
            };
            let raw_program = raw_programs.remove(index.unwrap_or_default());
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
