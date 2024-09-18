use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::Path,
    rc::Rc,
};

use anyhow::Result;
use serde::Serialize;
use uplc::{
    ast::NamedDeBruijn,
    machine::{indexed_term::IndexedTerm, Context, MachineState},
};

use crate::{chain_query::ChainQuery, uplc::LoadedProgram};

pub type Value = String;

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

pub async fn load_file(
    filename: &Path,
    parameters: &[String],
    query: ChainQuery,
) -> Result<Vec<LoadedProgram>> {
    println!("from file");
    let raw_programs = crate::uplc::load_programs_from_file(filename, query).await?;
    let mut programs = vec![];

    println!("{} program(s)", raw_programs.len());
    for raw_program in raw_programs {
        let arguments = parameters
            .iter()
            .enumerate()
            .map(|(index, param)| crate::uplc::parse_parameter(index, param.clone()))
            .collect::<Result<Vec<_>>>()?;
        let applied_program = crate::uplc::apply_parameters(raw_program, arguments)?;
        programs.push(applied_program);
    }
    println!("Done");
    Ok(programs)
}

pub struct RawFrame<'a> {
    pub label: &'a str,
    pub context: &'a Context,
    pub env: &'a Rc<Vec<uplc::machine::value::Value>>,
    pub term: &'a IndexedTerm<NamedDeBruijn>,
    pub ret_value: Option<&'a uplc::machine::value::Value>,
    pub location: Option<&'a String>,
    pub budget: ExBudget,
}

const MAX_CPU: i64 = 10000000000;
const MAX_MEM: i64 = 14000000;

pub fn parse_raw_frames<'a>(
    states: &'a [(MachineState, uplc::machine::cost_model::ExBudget)],
    source_map: &'a BTreeMap<u64, String>,
) -> Vec<RawFrame<'a>> {
    let mut frames = vec![];
    let mut prev_steps = 0;
    let mut prev_mem = 0;
    for (state, budget) in states {
        let (label, context, env, term, location, ret_value) = match state {
            MachineState::Compute(context, env, term) => {
                let prev_location = frames.last().and_then(|f: &RawFrame<'_>| f.location);
                (
                    "Compute",
                    context,
                    env,
                    term,
                    term.index()
                        .and_then(|i| source_map.get(&i))
                        .or(prev_location),
                    None,
                )
            }
            MachineState::Done(term) => {
                let prev_frame: &RawFrame =
                    frames.last().expect("Invalid program starts with return");
                (
                    "Done",
                    prev_frame.context,
                    prev_frame.env,
                    term,
                    term.index()
                        .and_then(|i| source_map.get(&i))
                        .or(prev_frame.location),
                    None,
                )
            }
            MachineState::Return(context, value) => {
                let prev_frame: &RawFrame =
                    frames.last().expect("Invalid program starts with return");
                (
                    "Return",
                    context,
                    prev_frame.env,
                    prev_frame.term,
                    prev_frame.location,
                    Some(value),
                )
            }
        };
        let steps = MAX_CPU - budget.cpu;
        let mem = MAX_MEM - budget.mem;
        frames.push(RawFrame {
            label,
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

pub fn parse_context(context: &Context) -> Vec<String> {
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

pub fn parse_env(env: &[uplc::machine::value::Value]) -> Vec<EnvVar> {
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

pub fn parse_uplc_value(value: uplc::machine::value::Value) -> Value {
    uplc::machine::discharge::value_as_term(value).to_string()
}

pub fn read_source_files(source_root: &Path, frames: &[RawFrame<'_>]) -> BTreeMap<String, String> {
    let filenames: BTreeSet<&str> = frames
        .iter()
        .filter_map(|f| f.location)
        .filter_map(|loc| loc.split_once(":"))
        .map(|(file, _)| file)
        .collect();

    let mut roots = vec![source_root.join("validators"), source_root.join("lib")];

    if let Ok(packages) = fs::read_dir(source_root.join("build").join("packages")) {
        for package in packages {
            if let Some(dir) = package
                .ok()
                .filter(|e| e.file_type().ok().is_some_and(|f| f.is_dir()))
            {
                roots.push(dir.path().join("lib"));
            }
        }
    }

    let mut files = BTreeMap::new();

    for filename in filenames {
        if let Some(contents) = roots
            .iter()
            .find_map(|root| fs::read_to_string(root.join(filename)).ok())
        {
            files.insert(filename.to_string(), contents);
        }
    }

    files
}
