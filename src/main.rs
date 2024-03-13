use uplc::machine::value::{Value};
use uplc::ast::{Constant};

fn main() {
    let n = Value::Con(Constant::Integer(1.into()).into());
    println!("Hello, world! {:?}", n);
}
