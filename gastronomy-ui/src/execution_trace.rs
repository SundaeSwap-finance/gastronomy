use std::{collections::BTreeMap, fmt::Display};

use gastronomy::{
    execution_trace::{parse_context, parse_env, parse_raw_frames, parse_uplc_value, RawFrame},
    uplc::{self, LoadedProgram, Program},
    Frame,
};
use pallas_codec::flat::Flat;
use tauri::InvokeError;
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
    pub async fn frame_count(&self) -> Result<usize, InvokeError> {
        let (frame_count_sink, frame_count_source) = oneshot::channel();
        let request = WorkerRequest::FrameCount(frame_count_sink);
        self.worker_channel
            .send(request)
            .await
            .map_err(to_invoke_error)?;
        frame_count_source.await.map_err(to_invoke_error)?
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
}

fn to_invoke_error<T: Display>(err: T) -> InvokeError {
    InvokeError::from(err.to_string())
}

type ResponseChannel<T> = oneshot::Sender<Result<T, InvokeError>>;

enum WorkerRequest {
    FrameCount(ResponseChannel<usize>),
    GetFrame(usize, ResponseChannel<Frame>),
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
                WorkerRequest::FrameCount(res) => {
                    let _ = res.send(Ok(frames.len()));
                }
                WorkerRequest::GetFrame(index, res) => {
                    let _ = res.send(Self::get_frame(index, &frames));
                }
            }
        }
    }

    fn get_frame(index: usize, frames: &[RawFrame<'_>]) -> Result<Frame, InvokeError> {
        let Some(raw) = frames.get(index) else {
            return Err(InvokeError::from("Invalid frame index"));
        };
        let frame = Frame {
            label: raw.label.to_string(),
            context: parse_context(raw.context),
            env: parse_env(raw.env),
            term: raw.term.to_string(),
            ret_value: raw.ret_value.map(|v| parse_uplc_value(v.clone())),
            location: raw.location.cloned(),
            budget: raw.budget.clone(),
        };
        Ok(frame)
    }
}
