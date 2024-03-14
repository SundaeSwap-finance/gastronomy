use uplc::ast::Constant;
use uplc::machine::value::Value;

fn main() {
    let n = Value::Con(Constant::Integer(1.into()).into());
    println!("Hello, world! {:?}", n);
}
