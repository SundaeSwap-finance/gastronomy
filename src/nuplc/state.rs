use super::context::Context;
use super::term::Term;
use super::env::Env;
use super::value::Value;

#[derive(Debug)]
pub enum State {
    Compute(Context, Env, Term),
    Return(Context, Value),
    Done(Term),
}

impl State {
    pub fn new(term: Term) -> State {
        State::Compute(Context::NoFrame, Env::new(), term)
    }
}