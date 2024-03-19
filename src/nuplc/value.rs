use std::rc::Rc;
use super::types::Name;
use super::constant::Constant;
use super::term::Term;
use super::env::Env;
use super::builtins::BuiltinEvaluation;
use num_bigint::BigInt;
use anyhow::*;

#[derive(Clone, Debug)]
pub enum Value {
    Constant(Rc<Constant>),
    Delay(Rc<Term>, Env),
    Lambda {
        parameter: Rc<Name>,
        body: Rc<Term>,
        env: Env,
    },
    Builtin(BuiltinEvaluation),
}

impl Value {
    pub fn as_integer(&self) -> Result<BigInt> {
        match self {
            Value::Constant(constant) => match constant.as_ref() {
                Constant::Integer(x) => Ok(x.clone()),
                _ => bail!("Expected integer"),
            },
            _ => bail!("Expected constant"),
        }
    }
}

impl Into<Term> for Value {
    fn into(self) -> Term {
        match self {
            Value::Constant(x) => Term::Constant(x),
            Value::Delay(body, _) => Term::Delay(body),
            Value::Lambda { parameter, body, .. } => Term::Lambda { parameter, body },
            Value::Builtin(x) => Term::Builtin(x.function),
        }
    }
}