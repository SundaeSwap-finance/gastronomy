use std::{collections::BTreeMap, path::PathBuf};

use anyhow::{Result, bail};
use app::App;
use clap::{Parser, Subcommand, command};
use figment::providers::Env;
use gastronomy::{
    chain_query::ChainQuery,
    compute_script_overrides,
    config::{Config, load_base_config},
    parse_script_overrides,
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
        #[clap(long)]
        source_root: Option<PathBuf>,
        /// A Cardano blueprint JSON file containing the overriding scripts, if applicable (defualts to plutus.json)
        #[clap(value_name = "FILEPATH")]
        blueprint: Option<PathBuf>,
        /// A mapping (colon-separated) from a script hash in the transaction to the script hash of another script found in the blueprint
        /// For example:`d27cee75:197c9353`
        /// *Only supported by transaction ID and transaction files*
        #[clap(long("script-override"), value_name = "FROM:TO:VERSION", num_args(0..), verbatim_doc_comment)]
        script_overrides: Vec<String>,
    },
}

fn load_config() -> Result<Config> {
    let config = load_base_config()
        .merge(Env::raw().ignore(&["BLOCKFROST"]).split("_"))
        .extract()?;
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
            source_root,
            blueprint,
            script_overrides,
        }) => {
            let overrides =
                compute_script_overrides(parse_script_overrides(script_overrides)?, blueprint)?;

            let mut raw_programs =
                gastronomy::uplc::load_programs_from_file(&file, query, overrides).await?;
            let index = index.or(if raw_programs.len() == 1 {
                None
            } else {
                Some(0)
            });
            if index.unwrap_or_default() >= raw_programs.len() {
                bail!(
                    "Invalid index #{}, tx only has {} redeemer(s)",
                    index.unwrap_or_default(),
                    raw_programs.len()
                );
            }
            let raw_program = raw_programs.remove(index.unwrap_or_default());
            let arguments = parameters
                .iter()
                .enumerate()
                .map(|(index, param)| gastronomy::uplc::parse_parameter(index, param.clone()))
                .collect::<Result<Vec<_>>>()?;
            let applied_program = gastronomy::uplc::apply_parameters(raw_program, arguments)?;
            let states = gastronomy::uplc::execute_program(applied_program.program)?;
            let frames =
                gastronomy::execution_trace::parse_raw_frames(&states, &applied_program.source_map);

            let source_files = if let Some(source_root) = source_root {
                gastronomy::execution_trace::read_source_files(&source_root, &frames)
            } else {
                BTreeMap::new()
            };
            let source_token_indices =
                gastronomy::execution_trace::find_source_token_indices(&frames);

            let mut terminal = utils::init()?;
            let mut app = App {
                file_name: file,
                index,
                cursor: 0,
                frames,
                source_files,
                source_token_indices,
                exit: false,
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
