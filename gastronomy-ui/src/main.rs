// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::path::{Path, PathBuf};

use api::{CreateTraceResponse, GetFrameResponse, GetSourceResponse, GetTraceSummaryResponse};
use dashmap::DashMap;
use execution_trace::ExecutionTrace;
use figment::providers::{Env, Serialized};
use gastronomy::{
    chain_query::ChainQuery,
    config::{load_base_config, Config},
};
use tauri::{InvokeError, Manager, State, Wry};
use tauri_plugin_store::{with_store, StoreBuilder, StoreCollection};

mod api;
mod execution_trace;

struct SessionState {
    traces: DashMap<String, ExecutionTrace>,
}

fn load_config(app_handle: &tauri::AppHandle) -> Result<Config, InvokeError> {
    let stores = app_handle.state::<StoreCollection<Wry>>();
    let path = PathBuf::from("settings.json");
    let saved_config = with_store(app_handle.clone(), stores, path, |store| {
        Ok(store.get("config").cloned())
    })?;
    let mut figment = load_base_config();
    if let Some(saved) = saved_config {
        figment = figment.merge(Serialized::defaults(saved));
    }
    let config = figment
        .merge(Env::raw().ignore(&["BLOCKFROST"]).split("_"))
        .extract()
        .map_err(|e| InvokeError::from(e.to_string()))?;
    Ok(config)
}

#[tauri::command]
async fn create_traces<'a>(
    file: &Path,
    parameters: Vec<String>,
    state: State<'a, SessionState>,
    app_handle: tauri::AppHandle,
) -> Result<CreateTraceResponse, InvokeError> {
    println!("Creating traces {:?} {:?}", file, parameters);

    let config = load_config(&app_handle)?;

    let query = if let Some(blockfrost) = &config.blockfrost {
        ChainQuery::blockfrost(blockfrost)
    } else {
        ChainQuery::None
    };

    let mut programs = gastronomy::execution_trace::load_file(file, &parameters, query)
        .await
        .map_err(InvokeError::from_anyhow)?;
    let mut identifiers = vec![];
    for program in programs.drain(..) {
        let trace = ExecutionTrace::from_program(program)?;
        let identifier = trace.identifier.clone();
        state.traces.insert(identifier.clone(), trace);
        identifiers.push(identifier);
    }
    Ok(CreateTraceResponse { identifiers })
}

#[tauri::command]
async fn get_trace_summary(
    identifier: &str,
    state: State<'_, SessionState>,
) -> Result<GetTraceSummaryResponse, InvokeError> {
    println!("Getting summary");
    let Some(trace) = state.traces.get(identifier) else {
        return Err(InvokeError::from("Trace not found"));
    };
    let (frame_count, source_token_indices) = trace.get_trace_summary().await?;
    Ok(GetTraceSummaryResponse {
        frame_count,
        source_token_indices,
    })
}

#[tauri::command]
async fn get_frame(
    identifier: &str,
    frame: usize,
    state: State<'_, SessionState>,
) -> Result<GetFrameResponse, InvokeError> {
    println!("Getting frame");
    let Some(trace) = state.traces.get(identifier) else {
        return Err(InvokeError::from("Trace not found"));
    };
    let frame = trace.get_frame(frame).await?;
    Ok(GetFrameResponse { frame })
}

#[tauri::command]
async fn get_source(
    identifier: &str,
    source_root: &Path,
    state: State<'_, SessionState>,
) -> Result<GetSourceResponse, InvokeError> {
    let Some(trace) = state.traces.get(identifier) else {
        return Err(InvokeError::from("Trace not found"));
    };
    let files = trace.read_source_files(source_root).await?;
    Ok(GetSourceResponse { files })
}

const STACK_SIZE: usize = 4 * 1024 * 1024;
fn main() {
    let runtime = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .thread_stack_size(STACK_SIZE)
        .build()
        .unwrap();
    tauri::async_runtime::set(runtime.handle().clone());

    tauri::Builder::default()
        .plugin(tauri_plugin_store::Builder::default().build())
        .setup(|app| {
            let mut store = StoreBuilder::new(app.handle(), "settings.json".parse()?).build();
            let load_res = store.load();
            if let Err(tauri_plugin_store::Error::Io(e)) = &load_res {
                if e.kind() == std::io::ErrorKind::NotFound {
                    return Ok(());
                }
            }
            load_res?;
            Ok(())
        })
        .manage(SessionState {
            traces: DashMap::new(),
        })
        .invoke_handler(tauri::generate_handler![
            create_traces,
            get_trace_summary,
            get_frame,
            get_source,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
