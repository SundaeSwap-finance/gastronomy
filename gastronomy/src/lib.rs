use std::path::Path;

use anyhow::Result;

pub use execution_trace::{ExecutionTrace, Frame};

mod execution_trace;
pub mod uplc;

pub fn greeting() -> &'static str {
    "Hello"
}

pub fn trace_execution(filename: &Path, parameters: &[String]) -> Result<ExecutionTrace> {
    ExecutionTrace::from_file(filename, parameters)
}
