use blake2b_simd::Hash;
use serde::{Deserialize, Serialize};
extern crate alloc;
pub const IOTA_ADDRESS_LENGTH: usize = 32;

#[derive(
    Debug, Eq, Default, PartialEq, Ord, PartialOrd, Copy, Clone, Hash, Serialize, Deserialize,
)]
pub struct ObjectID([u8; IOTA_ADDRESS_LENGTH]);

impl ObjectID {
    pub const fn new(obj_id: [u8; IOTA_ADDRESS_LENGTH]) -> Self {
        Self(obj_id)
    }
}

impl From<blake2b_simd::Hash> for ObjectID {
    fn from(hash: Hash) -> Self {
        Self::new(<[u8; 32]>::try_from(hash.as_bytes()).unwrap())
    }
}

#[derive(
    Eq, PartialEq, Ord, PartialOrd, Clone, Copy, Hash, Default, Debug, Serialize, Deserialize,
)]
pub struct SequenceNumber(u64);

impl SequenceNumber {
    pub fn new(sequence_number: u64) -> Self {
        Self(sequence_number)
    }
}

#[derive(
    Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, Debug,
)]
pub struct Digest(#[serde(with = "serde_bytes")] [u8; IOTA_ADDRESS_LENGTH]);

impl Digest {
    pub const ZERO: Self = Digest([0; IOTA_ADDRESS_LENGTH]);

    pub const fn new(digest: [u8; IOTA_ADDRESS_LENGTH]) -> Self {
        Self(digest)
    }
}

pub type ObjectRef = (ObjectID, SequenceNumber, Digest);

#[derive(Serialize, Deserialize, Debug, PartialEq, Hash, Eq, Clone, PartialOrd, Ord)]
pub struct StructTag {
    pub address: ObjectID,
    pub module: Identifier,
    pub name: Identifier,
    #[serde(rename = "type_args", alias = "type_params")]
    pub type_params: alloc::vec::Vec<TypeTag>,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Hash, Eq, Clone, PartialOrd, Ord)]
pub enum TypeTag {
    // alias for compatibility with old json serialized data.
    #[serde(rename = "bool", alias = "Bool")]
    Bool,
    #[serde(rename = "u8", alias = "U8")]
    U8,
    #[serde(rename = "u64", alias = "U64")]
    U64,
    #[serde(rename = "u128", alias = "U128")]
    U128,
    #[serde(rename = "address", alias = "Address")]
    Address,
    #[serde(rename = "signer", alias = "Signer")]
    Signer,
    #[serde(rename = "vector", alias = "Vector")]
    Vector(alloc::boxed::Box<TypeTag>),
    #[serde(rename = "struct", alias = "Struct")]
    Struct(alloc::boxed::Box<StructTag>),
    // NOTE: Added in bytecode version v6, do not reorder!
    #[serde(rename = "u16", alias = "U16")]
    U16,
    #[serde(rename = "u32", alias = "U32")]
    U32,
    #[serde(rename = "u256", alias = "U256")]
    U256,
}

#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct Identifier(alloc::boxed::Box<str>);

impl Identifier {
    pub fn new(identifier: alloc::boxed::Box<str>) -> Identifier {
        Self(identifier)
    }
}

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy, Serialize, Deserialize)]
pub enum ObjectArg {
    ImmOrOwnedObject(ObjectRef),
    SharedObject {
        id: ObjectID,
        initial_shared_version: SequenceNumber,
        mutable: bool,
    },
    Receiving(ObjectRef),
}

#[derive(Debug, PartialEq, Eq, Hash, Clone, Serialize, Deserialize)]
pub enum CallArg {
    Pure(alloc::vec::Vec<u8>),
    Object(ObjectArg),
}

#[derive(Debug, PartialEq, Eq, Hash, Clone, Serialize, Deserialize)]
pub struct ProgrammableMoveCall {
    pub package: ObjectID,
    pub module: Identifier,
    pub function: Identifier,
    pub type_arguments: alloc::vec::Vec<TypeTag>,
    pub arguments: alloc::vec::Vec<Argument>,
}

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy, Serialize, Deserialize)]
pub enum Argument {
    GasCoin,
    Input(u16),
    Result(u16),
    NestedResult(u16, u16),
}

#[derive(Debug, PartialEq, Eq, Hash, Clone, Serialize, Deserialize)]
pub enum Command {
    MoveCall(alloc::boxed::Box<ProgrammableMoveCall>),
    TransferObjects(alloc::vec::Vec<Argument>, Argument),
    SplitCoins(Argument, alloc::vec::Vec<Argument>),
    MergeCoins(Argument, alloc::vec::Vec<Argument>),
    Publish(
        alloc::vec::Vec<alloc::vec::Vec<u8>>,
        alloc::vec::Vec<ObjectID>,
    ),
    MakeMoveVec(Option<TypeTag>, alloc::vec::Vec<Argument>),
    Upgrade(
        alloc::vec::Vec<alloc::vec::Vec<u8>>,
        alloc::vec::Vec<ObjectID>,
        ObjectID,
        Argument,
    ),
}

#[derive(Debug, PartialEq, Eq, Hash, Clone, Serialize, Deserialize)]
pub struct ProgrammableTransaction {
    pub inputs: alloc::vec::Vec<CallArg>,
    pub commands: alloc::vec::Vec<Command>,
}

#[derive(Debug, PartialEq, Eq, Hash, Clone, Serialize, Deserialize)]
pub enum TransactionKind {
    ProgrammableTransaction1(ProgrammableTransaction),
    // Other Kinds not implemented
}

#[derive(Debug, PartialEq, Eq, Hash, Clone, Serialize, Deserialize)]
pub struct GasData {
    pub payment: alloc::vec::Vec<ObjectRef>,
    pub owner: ObjectID,
    pub price: u64,
    pub budget: u64,
}

pub type EpochId = u64;

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy, Serialize, Deserialize)]
pub enum TransactionExpiration {
    None,
    Epoch(EpochId),
}

#[derive(Debug, PartialEq, Eq, Hash, Clone, Serialize, Deserialize)]
pub struct TransactionDataV1 {
    pub kind: TransactionKind,
    pub sender: ObjectID,
    pub gas_data: GasData,
    pub expiration: TransactionExpiration,
}

#[derive(Debug, PartialEq, Eq, Hash, Clone, Serialize, Deserialize)]
pub enum TransactionData {
    V1(TransactionDataV1),
}

#[cfg(test)]
mod tests2 {
    use super::*;
    use base64::Engine;
    use base64::prelude::BASE64_STANDARD;
    use bcs::{from_bytes, to_bytes};

    use crate::transaction_types::TransactionKind::ProgrammableTransaction1;
    use crate::transaction_types::{
        GasData, ObjectID, ProgrammableTransaction, TransactionDataV1, TransactionExpiration,
    };

    #[tokio::test]
    async fn test_encode() {
        let tx = TransactionData::V1(TransactionDataV1 {
            expiration: TransactionExpiration::None,
            sender: ObjectID::new([
                150, 210, 207, 1, 112, 242, 209, 131, 20, 109, 36, 202, 26, 144, 76, 105, 214, 185,
                186, 117, 50, 228, 162, 206, 56, 106, 245, 177, 152, 64, 149, 150,
            ]),
            kind: ProgrammableTransaction1(ProgrammableTransaction {
                commands: alloc::vec![Command::MoveCall(Box::from(ProgrammableMoveCall {
                    package: ObjectID([0u8; 32]),
                    module: Identifier(Box::from("temperature_sensors")),
                    function: Identifier(Box::from("push_temperature_reading")),
                    type_arguments: vec![],
                    arguments: vec![
                        Argument::Input(0),
                        Argument::Input(1),
                        Argument::Input(2),
                        Argument::Input(3),
                        Argument::Input(4),
                    ]
                }))],
                inputs: alloc::vec![
                    CallArg::Pure(bcs::to_bytes(&123u8).unwrap()),
                    CallArg::Pure(bcs::to_bytes("ISS").unwrap()),
                    CallArg::Pure(bcs::to_bytes::<u32>(&32000).unwrap()),
                    CallArg::Pure(bcs::to_bytes::<u8>(&255).unwrap()),
                    CallArg::Pure(bcs::to_bytes::<u64>(&1).unwrap()),
                ],
            }),
            gas_data: GasData {
                budget: 99999,
                owner: ObjectID::new([
                    150, 210, 207, 1, 112, 242, 209, 131, 20, 109, 36, 202, 26, 144, 76, 105, 214,
                    185, 186, 117, 50, 228, 162, 206, 56, 106, 245, 177, 152, 64, 149, 150,
                ]),
                price: 8888,
                payment: alloc::vec![(ObjectID([8u8; 32]), SequenceNumber(3), Digest([9u8; 32]))],
            },
        });

        let b = to_bytes(&tx).unwrap();
        std::print!("{:#?}\n", tx);

        let mut output = [0u8; 1024];
        let size = BASE64_STANDARD.encode_slice(b, &mut output).unwrap();

        std::print!(
            "\nbase64: {}",
            alloc::str::from_utf8(&output[..size]).unwrap()
        );

        let mut tx_bytes = [0u8; 1024];
        let tx_size = BASE64_STANDARD
            .decode_slice(&output[..size], &mut tx_bytes)
            .unwrap();

        match from_bytes::<TransactionData>(&tx_bytes[..tx_size]) {
            Ok(s) => {
                std::print!("{:#?}", s)
            }
            Err(e) => {
                panic!("{:#?}", e)
            }
        }
    }

    #[tokio::test]
    async fn test_decode_from_unsafe_move_call() {
        let move_call = "AAAFAAF7AAsKTHVrYXMgSG9tZQAEAH0AAAAB/wAIAQAAAAAAAAABAMf/nu/aCgOsDfHUaZrN3TD+g8f0TeYhj/yWm9wg0xEOE3RlbXBlcmF0dXJlX3NlbnNvcnMYcHVzaF90ZW1wZXJhdHVyZV9yZWFkaW5nAAUBAAABAQABAgABAwABBACW0s8BcPLRgxRtJMoakExp1rm6dTLkos44avWxmECVlgEAjpZCZJz+EzMlxbhzVc5C7JFdI2OKnXuSaR12/N393gMAAAAAAAAAIK6FX5Rt9gNSKze0pLtiFWdqUtMNtvbWT4qUoKfIR31pltLPAXDy0YMUbSTKGpBMada5unUy5KLOOGr1sZhAlZboAwAAAAAAAADh9QUAAAAAAA==";
        let mut tx_bytes = [0u8; 1024];
        let tx_size = BASE64_STANDARD
            .decode_slice(move_call, &mut tx_bytes)
            .unwrap();
        std::print!("before match");

        match from_bytes::<crate::transaction_types::TransactionData>(&tx_bytes[..tx_size]) {
            Ok(s) => {
                std::print!("{:#?}", s)
            }
            Err(e) => {
                panic!("{:#?}", e)
            }
        }
    }
}
