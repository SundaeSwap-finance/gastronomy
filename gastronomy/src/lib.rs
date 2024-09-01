use std::path::Path;

use anyhow::Result;

use chain_query::ChainQuery;
pub use execution_trace::{ExecutionTrace, Frame};

pub mod chain_query;
mod execution_trace;
pub mod uplc;

pub async fn trace_executions(
    filename: &Path,
    parameters: &[String],
    query: impl ChainQuery,
) -> Result<Vec<ExecutionTrace>> {
    ExecutionTrace::from_file(filename, parameters, query).await
}
