use std::{collections::BTreeMap, path::Path};

use anyhow::Result;
use serde::Serialize;
use uplc::machine::{Context, MachineState};
use uuid::Uuid;

use crate::chain_query::ChainQuery;

pub type Value = String;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ExecutionTrace {
    pub identifier: String,
    pub filename: String,
    pub frames: Vec<Frame>,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Frame {
    pub label: String,
    pub context: Vec<String>,
    pub env: Vec<EnvVar>,
    pub term: Value,
    pub ret_value: Option<Value>,
    pub location: Option<String>,
    pub budget: ExBudget,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EnvVar {
    pub name: String,
    pub value: Value,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ExBudget {
    pub steps: i64,
    pub mem: i64,
    pub steps_diff: i64,
    pub mem_diff: i64,
}

impl ExecutionTrace {
    pub async fn from_file(
        filename: &Path,
        parameters: &[String],
        query: ChainQuery,
    ) -> Result<Vec<Self>> {
        println!("from file");
        let raw_programs = crate::uplc::load_programs_from_file(filename, query).await?;
        let mut execution_traces = vec![];

        println!("{} program(s)", raw_programs.len());
        for raw_program in raw_programs {
            let arguments = parameters
                .iter()
                .enumerate()
                .map(|(index, param)| crate::uplc::parse_parameter(index, param.clone()))
                .collect::<Result<Vec<_>>>()?;
            let applied_program = crate::uplc::apply_parameters(raw_program, arguments)?;
            let states = crate::uplc::execute_program(applied_program.program)?;
            let frames = parse_frames(&states, applied_program.source_map);
            execution_traces.push(Self {
                identifier: Uuid::new_v4().to_string(),
                filename: filename.display().to_string(),
                frames,
            })
        }
        println!("Done");
        Ok(execution_traces)
    }
}

const MAX_CPU: i64 = 10000000000;
const MAX_MEM: i64 = 14000000;

fn parse_frames(
    states: &[(MachineState, uplc::machine::cost_model::ExBudget)],
    source_map: BTreeMap<u64, String>,
) -> Vec<Frame> {
    let mut frames = vec![];
    let mut prev_steps = 0;
    let mut prev_mem = 0;
    for (state, budget) in states {
        let (label, context, env, term, location, ret_value) = match state {
            MachineState::Compute(context, env, term) => (
                "Compute",
                parse_context(context),
                parse_env(env),
                term.to_string(),
                term.index().and_then(|i| source_map.get(&i)).cloned(),
                None,
            ),
            MachineState::Done(term) => {
                let prev_frame: &Frame = frames.last().expect("Invalid program starts with return");
                (
                    "Done",
                    prev_frame.context.clone(),
                    prev_frame.env.clone(),
                    term.to_string(),
                    term.index().and_then(|i| source_map.get(&i)).cloned(),
                    None,
                )
            }
            MachineState::Return(context, value) => {
                let prev_frame: &Frame = frames.last().expect("Invalid program starts with return");
                let ret_value = parse_uplc_value(value.clone());
                (
                    "Return",
                    parse_context(context),
                    prev_frame.env.clone(),
                    prev_frame.term.clone(),
                    prev_frame.location.clone(),
                    Some(ret_value),
                )
            }
        };
        let steps = MAX_CPU - budget.cpu;
        let mem = MAX_MEM - budget.mem;
        frames.push(Frame {
            label: label.to_string(),
            context,
            env,
            term,
            ret_value,
            location,
            budget: ExBudget {
                steps,
                mem,
                steps_diff: steps - prev_steps,
                mem_diff: mem - prev_mem,
            },
        });
        prev_steps = steps;
        prev_mem = mem;
    }
    frames
}

fn parse_context(context: &Context) -> Vec<String> {
    let mut frames = vec![];
    let mut current = Some(context);
    while let Some(curr) = current {
        let (message, next) = parse_context_frame(curr);
        frames.push(message);
        current = next;
    }
    frames
}

fn parse_context_frame(context: &Context) -> (String, Option<&Context>) {
    match context {
        Context::FrameAwaitArg(_, next) => ("Get Function Argument".into(), Some(next)),
        Context::FrameAwaitFunTerm(_, _, next) => ("Get Function".into(), Some(next)),
        Context::FrameAwaitFunValue(_, next) => ("Evaluate Function".into(), Some(next)),
        Context::FrameCases(_, _, next) => ("Match Cases".into(), Some(next)),
        Context::FrameConstr(_, _, _, _, next) => ("Construct Data".into(), Some(next)),
        Context::FrameForce(next) => ("Force".into(), Some(next)),
        Context::NoFrame => ("Root".into(), None),
    }
}

fn parse_env(env: &[uplc::machine::value::Value]) -> Vec<EnvVar> {
    env.iter()
        .rev()
        .enumerate()
        .map(|(idx, v)| {
            let name = format!("i_{}", idx + 1);
            let value = parse_uplc_value(v.clone());
            EnvVar {
                name,
                value: value.to_string(),
            }
        })
        .collect()
}

fn parse_uplc_value(value: uplc::machine::value::Value) -> Value {
    uplc::machine::discharge::value_as_term(value).to_string()
}
