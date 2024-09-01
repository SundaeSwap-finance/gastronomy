// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::path::{Path, PathBuf};

use api::{CreateTraceResponse, GetFrameResponse, GetTraceSummaryResponse};
use dashmap::DashMap;
use gastronomy::{chain_query::Blockfrost, ExecutionTrace};
use tauri::{InvokeError, Manager, State, Wry};
use tauri_plugin_store::{with_store, StoreBuilder, StoreCollection};

mod api;

struct SessionState {
    traces: DashMap<String, ExecutionTrace>,
}

#[tauri::command]
async fn create_traces<'a>(
    file: &Path,
    parameters: Vec<String>,
    state: State<'a, SessionState>,
    app_handle: tauri::AppHandle,
    stores: State<'a, StoreCollection<Wry>>,
) -> Result<api::CreateTraceResponse, InvokeError> {
    println!("Creating traces {:?} {:?}", file, parameters);
    let path = PathBuf::from("settings.json");

    let key = with_store(app_handle, stores, path, |store| {
        let key: String = store
            .get("blockfrost.key")
            .and_then(|v| v.as_str())
            .map(|v| v.to_string())
            .unwrap_or_default();
        Ok(key)
    })?;
    let mut traces = gastronomy::trace_executions(file, &parameters, Blockfrost::new(key.as_str()))
        .await
        .unwrap();
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
            let _ = store.load();
            let stores = app.state::<StoreCollection<Wry>>();
            let path = PathBuf::from("settings.json");

            with_store(app.app_handle(), stores, path, |store| {
                let key: String = store
                    .get("blockfrost.key")
                    .and_then(|v| v.as_str())
                    .map(|v| v.to_string())
                    .unwrap_or_default();
                println!("Blockfrost key: {}", key);
                Ok(())
            })?;

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
