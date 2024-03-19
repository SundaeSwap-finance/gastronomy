
use std::rc::Rc;
use super::types::Name;
use super::constant::Constant;
use super::builtins::BuiltinFunction;

#[derive(Clone, Debug)]
pub enum Term {
    // tag: 0
    Var(Rc<Name>),
    // tag: 1
    Delay(Rc<Term>),
    // tag: 2
    Lambda {
        parameter: Rc<Name>,
        body: Rc<Term>,
    },
    // tag: 3
    Apply {
        function: Rc<Term>,
        argument: Rc<Term>,
    },
    // tag: 4
    Constant(Rc<Constant>),
    // tag: 5
    Force(Rc<Term>),
    // tag: 6
    Error,
    // tag: 7
    Builtin(BuiltinFunction),
    /*
    Constr {
        tag: usize,
        fields: Vec<Term>,
    },
    Case {
        constr: Rc<Term>,
        branches: Vec<Term>,
    },
    */
}
