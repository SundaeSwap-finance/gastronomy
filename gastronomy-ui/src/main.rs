// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::path::{Path, PathBuf};

use api::{CreateTraceResponse, GetFrameResponse, GetTraceSummaryResponse};
use dashmap::DashMap;
use figment::providers::{Env, Serialized};
use gastronomy::{
    chain_query::ChainQuery,
    config::{load_base_config, Config},
    ExecutionTrace,
};
use tauri::{InvokeError, Manager, State, Wry};
use tauri_plugin_store::{with_store, StoreBuilder, StoreCollection};

mod api;

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
        .merge(Env::raw())
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
) -> Result<api::CreateTraceResponse, InvokeError> {
    println!("Creating traces {:?} {:?}", file, parameters);

    let config = load_config(&app_handle)?;

    let query = if let Some(blockfrost) = &config.blockfrost {
        ChainQuery::blockfrost(blockfrost)
    } else {
        ChainQuery::None
    };

    let mut traces = gastronomy::trace_executions(file, &parameters, query)
        .await
        .map_err(InvokeError::from_anyhow)?;
    let mut identifiers = vec![];
    for trace in traces.drain(..) {
        let identifier = trace.identifier.clone();
        state.traces.insert(identifier.clone(), trace);
        identifiers.push(identifier);
    }
    Ok(CreateTraceResponse { identifiers })
}

#[tauri::command]
fn get_trace_summary(
    identifier: &str,
    state: State<SessionState>,
) -> Result<api::GetTraceSummaryResponse, InvokeError> {
    println!("Getting summary");
    let Some(trace) = state.traces.get(identifier) else {
        return Err(InvokeError::from("Trace not found"));
    };
    Ok(GetTraceSummaryResponse {
        frame_count: trace.frames.len(),
    })
}

#[tauri::command]
fn get_frame(
    identifier: &str,
    frame: usize,
    state: State<SessionState>,
) -> Result<api::GetFrameResponse, InvokeError> {
    println!("Getting frame");
    let Some(trace) = state.traces.get(identifier) else {
        return Err(InvokeError::from("Trace not found"));
    };
    let Some(frame) = trace.frames.get(frame) else {
        return Err(InvokeError::from("Frame not found"));
    };
    Ok(GetFrameResponse {
        frame: frame.clone(),
    })
}

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_store::Builder::default().build())
        .setup(|app| {
            let mut store = StoreBuilder::new(app.handle(), "settings.json".parse()?).build();
            store.load()?;

            Ok(())
        })
        .manage(SessionState {
            traces: DashMap::new(),
        })
        .invoke_handler(tauri::generate_handler![
            create_traces,
            get_trace_summary,
            get_frame
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
