use std::collections::HashMap;
use std::rc::Rc;
use super::types::Name;
use super::value::Value;
use std::fmt::{self, Debug, Formatter};

#[derive(Clone)]
pub struct Env {
    variables: HashMap<String, Value>,
}

impl Env {
    pub fn new() -> Env {
        Env {
            variables: HashMap::new(),
        }
    }

    pub fn get(&self, name: Rc<Name>) -> Value {
        self.variables.get(&name.name).unwrap().clone()
    }

    pub fn set(&self, name: Rc<Name>, value: Value) -> Env {
        let mut new_env = self.clone();
        new_env.variables.insert(name.name.clone(), value);
        new_env
    }
}

impl Debug for Env {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "Env(..)")
    }
}