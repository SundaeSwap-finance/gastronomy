use std::rc::Rc;
use num_bigint::BigInt;

#[derive(Debug)]

pub enum Constant {
    // tag: 0
    Integer(BigInt),
    // tag: 1
    ByteString(Vec<u8>),
    // tag: 2
    String(String),
    // tag: 3
    Unit,
    // tag: 4
    Bool(bool),
    // tag: 5
    ProtoList(Type, Vec<Constant>),
    // tag: 6
    ProtoPair(Type, Type, Rc<Constant>, Rc<Constant>),
    // tag: 7
    // Apply(Box<Constant>, Type),
    // tag: 8
    // Data(PlutusData),
    // Bls12_381G1Element(Box<blst::blst_p1>),
    // Bls12_381G2Element(Box<blst::blst_p2>),
    // Bls12_381MlResult(Box<blst::blst_fp12>),
}

#[derive(Debug)]
pub enum Type {
    Bool,
    Integer,
    String,
    ByteString,
    Unit,
    List(Rc<Type>),
    Pair(Rc<Type>, Rc<Type>),
    Data,
    /*
    Bls12_381G1Element,
    Bls12_381G2Element,
    Bls12_381MlResult,
    */
}