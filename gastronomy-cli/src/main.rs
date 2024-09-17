use std::path::PathBuf;

use anyhow::Result;
use app::App;
use clap::{command, Parser, Subcommand};
use figment::providers::Env;
use gastronomy::{
    chain_query::ChainQuery,
    config::{load_base_config, Config},
};

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
        #[clap(long)]
        index: Option<usize>,
    },
}

fn load_config() -> Result<Config> {
    let config = load_base_config().merge(Env::raw().split("_")).extract()?;
    Ok(config)
}

async fn run() -> Result<(), anyhow::Error> {
    utils::install_hooks().unwrap();

    let args = Args::parse();
    let config = load_config()?;

    let query = if let Some(blockfrost) = &config.blockfrost {
        ChainQuery::blockfrost(blockfrost)
    } else {
        ChainQuery::None
    };

    match args.command {
        Some(Commands::Run {
            file,
            parameters,
            index,
        }) => {
            let mut raw_programs = gastronomy::uplc::load_programs_from_file(&file, query).await?;
            let raw_program = raw_programs.remove(index.unwrap_or_default());
            let arguments = parameters
                .iter()
                .enumerate()
                .map(|(index, param)| gastronomy::uplc::parse_parameter(index, param.clone()))
                .collect::<Result<Vec<_>>>()?;
            let applied_program = gastronomy::uplc::apply_parameters(raw_program, arguments)?;
            let states = gastronomy::uplc::execute_program(applied_program.program)?;

            let mut terminal = utils::init()?;
            let mut app = App {
                file_name: file,
                cursor: 0,
                states,
                source_map: applied_program.source_map,
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

const STACK_SIZE: usize = 4 * 1024 * 1024;
fn main() -> Result<(), anyhow::Error> {
    std::thread::Builder::new()
        .stack_size(STACK_SIZE)
        .spawn(|| {
            tokio::runtime::Builder::new_multi_thread()
                .enable_all()
                .thread_stack_size(STACK_SIZE)
                .build()
                .unwrap()
                .block_on(run())
        })
        .unwrap()
        .join()
        .map_err(|err| anyhow::anyhow!("{:?}", err))?
}
