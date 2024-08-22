// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::path::Path;

use api::{CreateTraceResponse, GetFrameResponse, GetTraceSummaryResponse};
use dashmap::DashMap;
use gastronomy::ExecutionTrace;
use tauri::{InvokeError, State};

mod api;

struct SessionState {
    traces: DashMap<String, ExecutionTrace>,
}

#[tauri::command]
fn greet(name: &str) -> String {
    format!("{}, {}!", gastronomy::greeting(), name)
}

#[tauri::command]
fn create_trace(file: &Path, parameters: Vec<String>, state: State<SessionState>) -> Result<api::CreateTraceResponse, InvokeError> {
    let trace = gastronomy::trace_execution(file, &parameters).map_err(InvokeError::from_anyhow)?;
    let identifier = trace.identifier.clone();
    state.traces.insert(identifier.clone(), trace);
    Ok(CreateTraceResponse { identifier })
}

#[tauri::command]
fn get_trace_summary(identifier: &str, state: State<SessionState>) -> Result<api::GetTraceSummaryResponse, InvokeError> {
    let Some(trace) = state.traces.get(identifier) else {
        return Err(InvokeError::from("Trace not found"));
    };
    Ok(GetTraceSummaryResponse {
        frame_count: trace.frames.len(),
    })
}

#[tauri::command]
fn get_frame(identifier: &str, frame: usize, state: State<SessionState>) -> Result<api::GetFrameResponse, InvokeError> {
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
        .manage(SessionState { traces: DashMap::new() })
        .invoke_handler(tauri::generate_handler![greet, create_trace, get_trace_summary, get_frame])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
