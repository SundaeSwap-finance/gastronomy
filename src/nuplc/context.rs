use std::rc::Rc;
use std::fmt::{self, Debug, Formatter};
use super::{
    value::Value,
    env::Env,
    term::Term,
};

pub enum Context {
    ExpectingArgument(Value, Box<Context>),
    ExpectingFunction(Env, Term, Box<Context>),
    Forcing(Box<Context>),
    NoFrame,    
}
impl Debug for Context {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            Context::ExpectingArgument(_, _) => write!(f, "ExpectingArgument(..)"),
            Context::ExpectingFunction(_, _, _) => write!(f, "ExpectingFunction(..)"),
            Context::Forcing(_) => write!(f, "Forcing(..)"),
            Context::NoFrame => write!(f, "NoFrame"),
        }
    }
}

impl Context {
    pub fn expecting_argument(self, value: Value) -> Self {
        Context::ExpectingArgument(value, self.into())
    }
    pub fn forcing(self) -> Self {
        Context::Forcing(self.into())
    }
    pub fn expecting_function(self, env: &Env, argument: Rc<Term>) -> Self {
        Context::ExpectingFunction(env.clone(), argument.as_ref().clone(), self.into())
    }
    /*
    pub fn cases(self, env: &Env, branches: Vec<Term>) -> Self {
        Context::Cases(env.clone(), branches, self.into())
    }
    pub fn constructing(self, env: &Env, tag: usize, fields: Vec<Term>, values: Vec<Value>) -> Self {
        Context::Constructing(env.clone(), tag, fields, values, self.into())
    }
    */
}