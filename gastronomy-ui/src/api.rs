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
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GetFrameResponse {
    pub frame: Frame,
}
