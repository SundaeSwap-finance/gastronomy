pub mod builtins;
pub mod constant;
pub mod term;
pub mod types;
pub mod state;
pub mod context;
pub mod env;
pub mod value;

use state::{State, State::*};
use context::{Context, Context::*};
use env::Env;
use term::{Term, Term::*};
use value::Value;

use anyhow::*;

pub struct Machine {
}

impl Machine {
    pub fn new() -> Machine {
        Machine {}
    }

    pub fn step(&mut self, state: State) -> Result<State> {
        let new_state = match state {
            Compute(context, env, term) => self.step_compute(context, env, term)?,
            Return(context, value)      => self.step_return(context, value)?,
            Done(term)                  => State::Done(term),
        };
        Ok(new_state)
    }

    pub fn step_compute(&mut self, context: Context, env: Env, term: Term) -> Result<State> {
        let new_state = match term {
            Error                         => bail!("Explicit Error"),
            Var(name)                     => Return( context, env.get(name)                                                     ),
            Delay(body)                   => Return( context, Value::Delay(body, env)                                           ),
            Constant(x)                   => Return( context, Value::Constant(x)                                                ),
            Builtin(function)             => Return( context, Value::Builtin(function.clone().into())                           ),
            Lambda { parameter, body }    => Return( context, Value::Lambda { parameter, body, env }                            ),
            Force(body)                   => Compute(context.forcing(),                               env, body.as_ref().clone()     ),
            Apply { function, argument }  => Compute(context.expecting_function(&env, argument),      env, function.as_ref().clone() ),
        };
        Ok(new_state)
    }

    pub fn step_return(&mut self, context: Context, value: Value) -> Result<State> {
        let new_state = match context {
            NoFrame                                  => Done(value.into()),
            ExpectingFunction(arg_env, arg, context) => Compute(context.expecting_argument(value), arg_env, arg),
            Forcing(context)                         => self.apply_force(*context, value)?,
            ExpectingArgument(fun, context)          => self.evaluate_function(*context, fun, value)?,
        };
        Ok(new_state)
    }

    pub fn apply_force(&self, context: Context, value: Value) -> Result<State> {
        let new_state = match value {
            Value::Delay(body, env)    => Compute(context, env, body.as_ref().clone()),
            Value::Builtin(evaluation) => Return(context, evaluation.force()?),
            _                          => bail!("non-polymorphic instantiation"),
        };
        Ok(new_state)
    }

    pub fn evaluate_function(&self, context: Context, function: Value, argument: Value) -> Result<State> {
        let new_state = match function {
            Value::Lambda { parameter, body, env } => Compute(context, env.set(parameter, argument.clone()), body.as_ref().clone()),
            Value::Builtin(evaluation)             => Return(context, evaluation.apply(argument)?),
            _                                      => bail!("Try to evaluate something that's not a function"),
        };
        Ok(new_state)
    }
}

#[cfg(test)]
mod tests {
    use std::rc::Rc;
    use super::*;
    use super::types::*;

    #[test]
    fn test_machine() {
        let term = Apply {
            function: Rc::new(Lambda {
                parameter: Name { name: "x".to_string(), debruijn: DeBruijn(0), unique: Unique(0) }.into(),
                body: Var(Name { name: "x".to_string(), debruijn: DeBruijn(0), unique: Unique(0) }.into()).into(),
            }),
            argument: Constant(constant::Constant::Integer(42.into()).into()).into(),
        };
        let state = State::new(term);
        let mut machine = Machine::new();
        let state = machine.step(state).unwrap();
        println!("{:?}", state);
        let state = machine.step(state).unwrap();
        println!("{:?}", state);
        let state = machine.step(state).unwrap();
        println!("{:?}", state);
        let state = machine.step(state).unwrap();
        println!("{:?}", state);
        let state = machine.step(state).unwrap();
        println!("{:?}", state);
        let state = machine.step(state).unwrap();
        println!("{:?}", state);
        let state = machine.step(state).unwrap();
        println!("{:?}", state);
        panic!();
    }
}