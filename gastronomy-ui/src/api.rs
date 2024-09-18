use std::collections::BTreeMap;

use gastronomy::Frame;
use serde::Serialize;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateTraceResponse {
    pub identifiers: Vec<String>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GetTraceSummaryResponse {
    pub frame_count: usize,
    pub source_token_indices: Vec<usize>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GetFrameResponse {
    pub frame: Frame,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GetSourceResponse {
    pub files: BTreeMap<String, String>,
}
