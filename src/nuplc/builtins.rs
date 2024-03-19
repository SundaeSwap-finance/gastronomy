use std::rc::Rc;
use super::value::Value;
use anyhow::*;
use super::constant::Constant;
use num_bigint::BigInt;

#[derive(Clone, Debug)]
pub enum BuiltinFunction {
    // Integer functions
    AddInteger = 0,
    SubtractInteger = 1,
    MultiplyInteger = 2,
    DivideInteger = 3,
    QuotientInteger = 4,
    RemainderInteger = 5,
    ModInteger = 6,
    EqualsInteger = 7,
    LessThanInteger = 8,
    LessThanEqualsInteger = 9,
    // ByteString functions
    AppendByteString = 10,
    ConsByteString = 11,
    SliceByteString = 12,
    LengthOfByteString = 13,
    IndexByteString = 14,
    EqualsByteString = 15,
    LessThanByteString = 16,
    LessThanEqualsByteString = 17,
    // Cryptography and hash functions
    Sha2_256 = 18,
    Sha3_256 = 19,
    Blake2b_256 = 20,
    Keccak_256 = 71,
    Blake2b_224 = 72,
    VerifyEd25519Signature = 21,
    VerifyEcdsaSecp256k1Signature = 52,
    VerifySchnorrSecp256k1Signature = 53,
    // String functions
    AppendString = 22,
    EqualsString = 23,
    EncodeUtf8 = 24,
    DecodeUtf8 = 25,
    // Bool function
    IfThenElse = 26,
    // Unit function
    ChooseUnit = 27,
    // Tracing function
    Trace = 28,
    // Pairs functions
    FstPair = 29,
    SndPair = 30,
    // List functions
    ChooseList = 31,
    MkCons = 32,
    HeadList = 33,
    TailList = 34,
    NullList = 35,
    // Data functions
    // It is convenient to have a "choosing" function for a data type that has more than two
    // constructors to get pattern matching over it and we may end up having multiple such data
    // types, hence we include the name of the data type as a suffix.
    ChooseData = 36,
    ConstrData = 37,
    MapData = 38,
    ListData = 39,
    IData = 40,
    BData = 41,
    UnConstrData = 42,
    UnMapData = 43,
    UnListData = 44,
    UnIData = 45,
    UnBData = 46,
    EqualsData = 47,
    SerialiseData = 51,
    // Misc constructors
    // Constructors that we need for constructing e.g. Data. Polymorphic builtin
    // constructors are often problematic (See note [Representable built-in
    // functions over polymorphic built-in types])
    MkPairData = 48,
    MkNilData = 49,
    MkNilPairData = 50,

    // Bitwise
    IntegerToByteString = 73,
    ByteStringToInteger = 74,
}

impl BuiltinFunction {
    pub fn needed_arguments(&self) -> usize {
        use BuiltinFunction::*;
        match self {
            AddInteger => 2,
            SubtractInteger => 2,
            MultiplyInteger => 2,
            DivideInteger => 2,
            QuotientInteger => 2,
            RemainderInteger => 2,
            ModInteger => 2,
            EqualsInteger => 2,
            LessThanInteger => 2,
            LessThanEqualsInteger => 2,
            AppendByteString => 2,
            ConsByteString => 2,
            SliceByteString => 3,
            LengthOfByteString => 1,
            IndexByteString => 1,
            EqualsByteString => 2,
            LessThanByteString => 2,
            LessThanEqualsByteString => 2,
            Sha2_256 => 1,
            Sha3_256 => 1,
            Blake2b_256 => 1,
            Keccak_256 => 1,
            Blake2b_224 => 1,
            VerifyEd25519Signature => 3,
            VerifyEcdsaSecp256k1Signature => 3,
            VerifySchnorrSecp256k1Signature => 3,
            AppendString => 2,
            EqualsString => 2,
            EncodeUtf8 => 1,
            DecodeUtf8 => 1,
            IfThenElse => 3,
            ChooseUnit => 2,
            Trace => 2,
            FstPair => 1,
            SndPair => 1,
            ChooseList => 3,
            MkCons => 0,
            HeadList => 1,
            TailList => 2,
            NullList => 1,
            // It is convenient to have a "choosing" function for a data type that has => 0,
            // constructors to get pattern matching over it and we may end up having => 0,
            // types, hence we include the name of the data type as => 0,
            ChooseData => 6,
            ConstrData => 2,
            MapData => 1,
            ListData => 1,
            IData => 1,
            BData => 1,
            UnConstrData => 1,
            UnMapData => 1,
            UnListData => 1,
            UnIData => 1,
            UnBData => 1,
            EqualsData => 2,
            SerialiseData => 1,
            // Constructors that we need for constructing e.g. Data=> 0,
            // constructors are often problematic (See note [Representable => 0,
            // functions over polymorphic built-=> 0,
            MkPairData => 2,
            MkNilData => 0,
            MkNilPairData => 0,
            IntegerToByteString => 3,
            ByteStringToInteger => 2,
        }
    }

    pub fn eval(self, args: Vec<Value>) -> Result<Value> {
        use BuiltinFunction::*;
        let constant = match self {
            AddInteger => {
                let a = args[0].as_integer()?;
                let b = args[1].as_integer()?;
                Constant::Integer(a + b)
            },
            SubtractInteger => {
                let a = args[0].as_integer()?;
                let b = args[1].as_integer()?;
                Constant::Integer(a - b)
            },
            _ => todo!(),
            /*
            MultiplyInteger => 2,
            DivideInteger => 2,
            QuotientInteger => 2,
            RemainderInteger => 2,
            ModInteger => 2,
            EqualsInteger => 2,
            LessThanInteger => 2,
            LessThanEqualsInteger => 2,
            AppendByteString => 2,
            ConsByteString => 2,
            SliceByteString => 3,
            LengthOfByteString => 1,
            IndexByteString => 1,
            EqualsByteString => 2,
            LessThanByteString => 2,
            LessThanEqualsByteString => 2,
            Sha2_256 => 1,
            Sha3_256 => 1,
            Blake2b_256 => 1,
            Keccak_256 => 1,
            Blake2b_224 => 1,
            VerifyEd25519Signature => 3,
            VerifyEcdsaSecp256k1Signature => 3,
            VerifySchnorrSecp256k1Signature => 3,
            AppendString => 2,
            EqualsString => 2,
            EncodeUtf8 => 1,
            DecodeUtf8 => 1,
            IfThenElse => 3,
            ChooseUnit => 2,
            Trace => 2,
            FstPair => 1,
            SndPair => 1,
            ChooseList => 3,
            MkCons => 0,
            HeadList => 1,
            TailList => 2,
            NullList => 1,
            // It is convenient to have a "choosing" function for a data type that has => 0,
            // constructors to get pattern matching over it and we may end up having => 0,
            // types, hence we include the name of the data type as => 0,
            ChooseData => 6,
            ConstrData => 2,
            MapData => 1,
            ListData => 1,
            IData => 1,
            BData => 1,
            UnConstrData => 1,
            UnMapData => 1,
            UnListData => 1,
            UnIData => 1,
            UnBData => 1,
            EqualsData => 2,
            SerialiseData => 1,
            // Constructors that we need for constructing e.g. Data=> 0,
            // constructors are often problematic (See note [Representable => 0,
            // functions over polymorphic built-=> 0,
            MkPairData => 2,
            MkNilData => 0,
            MkNilPairData => 0,
            IntegerToByteString => 3,
            ByteStringToInteger => 2,
            */
        };
        Ok(Value::Constant(Rc::new(constant)))
    }
}

#[derive(Clone, Debug)]
pub struct BuiltinEvaluation {
    pub arguments: Vec<Value>,
    pub function: BuiltinFunction,
    pub forces_remaining: u32,
}

impl BuiltinEvaluation {
    pub fn force(self) -> Result<Value> {
        let fully_applied = self.arguments.len() == self.function.needed_arguments();
        let value = match (self.forces_remaining, fully_applied) {
            (0, _) => bail!("Too many forces"),
            (1, true) => self.function.eval(self.arguments)?,
            _ => Value::Builtin(BuiltinEvaluation { forces_remaining: self.forces_remaining - 1, ..self }),
        };
        Ok(value)
    }

    pub fn apply(self, argument: Value) -> Result<Value> {
        let function = self.function.clone();
        let next = BuiltinEvaluation{ arguments: [&self.arguments[..], &[argument]].concat(), ..self };
        let fully_applied = next.arguments.len() == function.needed_arguments();
        let value = match (self.forces_remaining, fully_applied) {
            (0, true)  => function.eval(next.arguments)?,
            (0, false) => Value::Builtin(next),
            _ => bail!("Unexpected argument")
        };
        Ok(value)
    }
}

impl Into<BuiltinEvaluation> for BuiltinFunction {
    fn into(self) -> BuiltinEvaluation {
        BuiltinEvaluation {
            arguments: vec![],
            function: self,
            forces_remaining: 0,
        }
    }
}