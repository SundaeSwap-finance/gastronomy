// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::path::Path;

use api::{CreateTraceResponse, GetFrameResponse, GetSourceResponse, GetTraceSummaryResponse};
use dashmap::DashMap;
use execution_trace::ExecutionTrace;
use figment::providers::{Env, Serialized};
use gastronomy::{
    chain_query::ChainQuery,
    config::{Config, load_base_config},
    uplc::ScriptOverride,
};
use tauri::{State, ipc::InvokeError};
use tauri_plugin_store::StoreExt;

mod api;
mod execution_trace;

struct SessionState {
    traces: DashMap<String, ExecutionTrace>,
}

fn load_config(app_handle: &tauri::AppHandle) -> Result<Config, InvokeError> {
    let saved_config = app_handle
        .get_store("settings.json")
        .and_then(|s| s.get("config"));
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
async fn create_traces(
    file: &Path,
    parameters: Vec<String>,
    state: State<'_, SessionState>,
    app_handle: tauri::AppHandle,
) -> Result<CreateTraceResponse, InvokeError> {
    println!("Creating traces {:?} {:?}", file, parameters);

    let config = load_config(&app_handle)?;

    let query = if let Some(blockfrost) = &config.blockfrost {
        ChainQuery::blockfrost(blockfrost)
    } else {
        ChainQuery::None
    };

    let script_overrides = if let Some(script_overrides) = config.script_overrides {
        script_overrides
            .into_iter()
            .map(ScriptOverride::try_from)
            .collect::<Result<Vec<_>, _>>()
            .map_err(InvokeError::from_anyhow)?
    } else {
        Vec::new()
    };

    let mut programs =
        gastronomy::execution_trace::load_file(file, &parameters, query, script_overrides)
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
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_store::Builder::new().build())
        .setup(|app| {
            let store = app.store("settings.json")?;
            let load_res = store.reload();
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
