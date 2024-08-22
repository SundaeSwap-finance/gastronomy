use std::path::PathBuf;

use gastronomy::Frame;
use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
pub struct CreateTraceRequest {
    pub file: PathBuf,
    pub parameters: Vec<String>,
}

#[derive(Serialize)]
pub struct CreateTraceResponse {
    pub identifier: String,
}

#[derive(Deserialize)]
pub struct GetTraceSummaryRequest {
    pub identifier: String,
}

#[derive(Serialize)]
pub struct GetTraceSummaryResponse {
    pub frame_count: usize,
}

#[derive(Deserialize)]
pub struct GetFrameRequest {
    pub identifier: String,
    pub frame: usize,
}

#[derive(Serialize)]
pub struct GetFrameResponse {
    pub frame: Frame,
}