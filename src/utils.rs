use std::io::{self, stdout, Stdout};
use std::panic;
use std::rc::Rc;

use color_eyre::{config::HookBuilder, eyre};
use crossterm::execute;
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use ratatui::prelude::*;
use uplc::machine::value::Value;
use uplc::machine::Context;

pub type Tui = Terminal<CrosstermBackend<Stdout>>;

pub fn install_hooks() -> color_eyre::Result<()> {
    let (panic_hook, eyre_hook) = HookBuilder::default().into_hooks();

    // convert from a color_eyre PanicHook to a standard panic hook
    let panic_hook = panic_hook.into_panic_hook();
    panic::set_hook(Box::new(move |panic_info| {
        restore().unwrap();
        panic_hook(panic_info);
    }));

    // convert from a color_eyre EyreHook to a eyre ErrorHook
    let eyre_hook = eyre_hook.into_eyre_hook();
    eyre::set_hook(Box::new(
        move |error: &(dyn std::error::Error + 'static)| {
            restore().unwrap();
            eyre_hook(error)
        },
    ))?;

    Ok(())
}

pub fn init() -> io::Result<Tui> {
    execute!(stdout(), EnterAlternateScreen)?;
    enable_raw_mode()?;
    Terminal::new(CrosstermBackend::new(stdout()))
}

pub fn restore() -> io::Result<()> {
    execute!(stdout(), LeaveAlternateScreen)?;
    disable_raw_mode()?;
    Ok(())
}

pub fn env_to_string(env: &Rc<Vec<Value>>) -> String {
    let mut result = String::new();
    for (idx, v) in env.iter().rev().enumerate() {
        if idx > 0 {
            result.push_str("\n");
        }
        result.push_str(
            format!(
                "{}: {}",
                format!("i_{}", idx + 1).blue(),
                uplc::machine::discharge::value_as_term(v.clone())
            )
            .as_str(),
        );
    }
    return result;
}

pub fn context_to_string(context: Context) -> String {
    let mut result = String::new();
    do_context_to_string(&context, &mut result);
    return result;
}

pub fn do_context_to_string(context: &Context, so_far: &mut String) {
    let next = match context {
        Context::FrameAwaitArg(_, next) => {
            so_far.push_str("Get Function Argument");
            Some(next)
        }
        Context::FrameAwaitFunTerm(_, _, next) => {
            so_far.push_str("Get Function");
            Some(next)
        }
        Context::FrameAwaitFunValue(_, next) => {
            so_far.push_str("Evaluate Function");
            Some(next)
        }
        Context::FrameForce(next) => {
            so_far.push_str("Force");
            Some(next)
        }
        Context::FrameConstr(_, _, _, _, next) => {
            so_far.push_str("Construct Data");
            Some(next)
        }
        Context::FrameCases(_, _, next) => {
            so_far.push_str("Match Cases");
            Some(next)
        }
        Context::NoFrame => {
            so_far.push_str("Root");
            None
        }
    };
    if let Some(next) = next {
        so_far.push_str("\n -> ");
        do_context_to_string(next, so_far);
    }
}
