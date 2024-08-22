use std::path::Path;

use anyhow::Result;
use serde::Serialize;
use uplc::machine::{Context, MachineState};
use uuid::Uuid;

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
    pub cpu: i64,
    pub mem: i64,
}

impl ExecutionTrace {
    pub fn from_file(filename: &Path, parameters: &[String]) -> Result<Self> {
        let states = crate::uplc::execute_program(filename, parameters)?;
        let frames = parse_frames(&states);
        Ok(Self {
            identifier: Uuid::new_v4().to_string(),
            filename: filename.display().to_string(),
            frames,
        })
    }
}

fn parse_frames(states: &[(MachineState, uplc::machine::cost_model::ExBudget)]) -> Vec<Frame> {
    let mut frames = vec![];
    for (state, budget) in states {
        let (label, context, env, term, ret_value) = match state {
            MachineState::Compute(context, env, term) => (
                "Compute",
                parse_context(context),
                parse_env(env),
                term.to_string(),
                None,
            ),
            MachineState::Done(term) => {
                let prev_frame: &Frame = frames.last().expect("Invalid program starts with return");
                (
                    "Done",
                    prev_frame.context.clone(),
                    prev_frame.env.clone(),
                    term.to_string(),
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
                    Some(ret_value),
                )
            }
        };
        frames.push(Frame {
            label: label.to_string(),
            context,
            env,
            term,
            ret_value,
            budget: ExBudget {
                cpu: budget.cpu,
                mem: budget.mem,
            },
        })
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
    uplc::machine::discharge::value_as_term(value.clone()).to_string()
}
