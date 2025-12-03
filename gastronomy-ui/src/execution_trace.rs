use std::{
    collections::BTreeMap,
    fmt::Display,
    path::{Path, PathBuf},
};

use gastronomy::{
    Frame,
    execution_trace::{
        RawFrame, find_source_token_indices, parse_context, parse_env, parse_raw_frames,
        parse_uplc_value, read_source_files,
    },
    uplc::{self, LoadedProgram, Program},
};
use pallas_codec::flat::Flat;
use tauri::ipc::InvokeError;
use tokio::sync::{mpsc, oneshot};
use uuid::Uuid;

pub struct ExecutionTrace {
    pub identifier: String,
    worker_channel: mpsc::Sender<WorkerRequest>,
}

impl ExecutionTrace {
    pub fn from_program(program: LoadedProgram) -> Result<Self, InvokeError> {
        let identifier = Uuid::new_v4().to_string();

        // The Aiken uplc crate uses lots of Rc<T> internally, so it's not Send.
        // The string representation of a frame of execution can get HUGE, so we need to serialize it lazily.
        // So, send the raw bytes to another thread, and interact with it over a channel.
        let (worker_channel, requests) = mpsc::channel(16);
        let worker = ExecutionTraceWorker {
            raw_program: program.program.to_flat().map_err(to_invoke_error)?,
            source_map: program.source_map,
            requests,
        };
        std::thread::Builder::new()
            .name(identifier.clone())
            .stack_size(4 * 1024 * 1024)
            .spawn(|| worker.run())
            .map_err(to_invoke_error)?;

        Ok(Self {
            identifier,
            worker_channel,
        })
    }
    pub async fn get_trace_summary(&self) -> Result<(usize, Vec<usize>), InvokeError> {
        let (summary_sink, summary_source) = oneshot::channel();
        let request = WorkerRequest::GetTraceSummary(summary_sink);
        self.worker_channel
            .send(request)
            .await
            .map_err(to_invoke_error)?;
        summary_source.await.map_err(to_invoke_error)?
    }
    pub async fn get_frame(&self, frame: usize) -> Result<Frame, InvokeError> {
        let (frame_sink, frame_source) = oneshot::channel();
        let request = WorkerRequest::GetFrame(frame, frame_sink);
        self.worker_channel
            .send(request)
            .await
            .map_err(to_invoke_error)?;
        frame_source.await.map_err(to_invoke_error)?
    }
    pub async fn read_source_files(
        &self,
        source_root: &Path,
    ) -> Result<BTreeMap<String, String>, InvokeError> {
        let (files_sink, files_source) = oneshot::channel();
        let request = WorkerRequest::ReadSourceFiles(source_root.to_path_buf(), files_sink);
        self.worker_channel
            .send(request)
            .await
            .map_err(to_invoke_error)?;
        files_source.await.map_err(to_invoke_error)?
    }
}

fn to_invoke_error<T: Display>(err: T) -> InvokeError {
    InvokeError::from(err.to_string())
}

type ResponseChannel<T> = oneshot::Sender<Result<T, InvokeError>>;

enum WorkerRequest {
    GetTraceSummary(ResponseChannel<(usize, Vec<usize>)>),
    GetFrame(usize, ResponseChannel<Frame>),
    ReadSourceFiles(PathBuf, ResponseChannel<BTreeMap<String, String>>),
}

struct ExecutionTraceWorker {
    raw_program: Vec<u8>,
    source_map: BTreeMap<u64, String>,
    requests: mpsc::Receiver<WorkerRequest>,
}

impl ExecutionTraceWorker {
    fn run(self) {
        let program = Program::unflat(&self.raw_program).unwrap();
        let states = uplc::execute_program(program).unwrap();
        let frames = parse_raw_frames(&states, &self.source_map);

        let mut requests = self.requests;
        while let Some(request) = requests.blocking_recv() {
            match request {
                WorkerRequest::GetTraceSummary(res) => {
                    let _ = res.send(Self::get_trace_summary(&frames));
                }
                WorkerRequest::GetFrame(index, res) => {
                    let _ = res.send(Self::get_frame(index, &frames));
                }
                WorkerRequest::ReadSourceFiles(source_root, res) => {
                    let _ = res.send(Self::read_source_files(&source_root, &frames));
                }
            }
        }
    }

    fn get_trace_summary(frames: &[RawFrame<'_>]) -> Result<(usize, Vec<usize>), InvokeError> {
        let frame_count = frames.len();
        let source_token_indices = find_source_token_indices(frames);
        Ok((frame_count, source_token_indices))
    }

    fn get_frame(index: usize, frames: &[RawFrame<'_>]) -> Result<Frame, InvokeError> {
        let Some(raw) = frames.get(index) else {
            return Err(InvokeError::from("Invalid frame index"));
        };
        let frame = Frame {
            label: raw.label.to_string(),
            context: parse_context(raw.context),
            env: parse_env(&raw.env),
            term: raw.term.to_string(),
            ret_value: raw.ret_value.map(|v| parse_uplc_value(v.clone())),
            location: raw.location.cloned(),
            budget: raw.budget.clone(),
        };
        Ok(frame)
    }

    fn read_source_files(
        source_root: &Path,
        frames: &[RawFrame<'_>],
    ) -> Result<BTreeMap<String, String>, InvokeError> {
        Ok(read_source_files(source_root, frames))
    }
}
