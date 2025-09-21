use alloc::vec;
use libs::transaction_types::{
    Argument, CallArg, Command, GasData, Identifier, ObjectID, ObjectRef, ProgrammableMoveCall,
    ProgrammableTransaction, TransactionData, TransactionDataV1, TransactionExpiration,
    TransactionKind,
};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Debug, Clone, Deserialize)]
pub struct SensorReading<T: Serialize> {
    pub sensor_id: u8,
    pub location: heapless::String<256>,
    pub battery_reading: u8,
    pub data: T,
}

#[derive(Serialize, Debug, Clone, Deserialize)]
pub struct TemperatureReading {
    pub temperature: u32,
}

pub fn build_temperature_sensor_tx(
    sponsor_address: ObjectID,
    gas_coin: ObjectRef,
    sender_address: ObjectID,
    package_id: ObjectID,
    gas_budget: u64,
    reading: SensorReading<TemperatureReading>,
) -> TransactionData {
    TransactionData::V1(TransactionDataV1 {
        expiration: TransactionExpiration::None,
        sender: sender_address,
        kind: TransactionKind::ProgrammableTransaction1(ProgrammableTransaction {
            commands: vec![Command::MoveCall(
                alloc::boxed::Box::<ProgrammableMoveCall>::new(ProgrammableMoveCall {
                    package: package_id,
                    module: Identifier::new(alloc::boxed::Box::from("temperature")),
                    function: Identifier::new(alloc::boxed::Box::from("push_reading")),
                    type_arguments: vec![],
                    arguments: vec![
                        Argument::Input(0),
                        Argument::Input(1),
                        Argument::Input(2),
                        Argument::Input(3),
                    ],
                }),
            )],
            inputs: vec![
                CallArg::Pure(bcs::to_bytes(&reading.sensor_id).unwrap()),
                CallArg::Pure(bcs::to_bytes(&reading.location).unwrap()),
                CallArg::Pure(bcs::to_bytes(&reading.battery_reading).unwrap()),
                CallArg::Pure(bcs::to_bytes(&reading.data.temperature).unwrap()),
            ],
        }),
        gas_data: GasData {
            budget: gas_budget,
            owner: sponsor_address,
            price: 1000,
            payment: vec![gas_coin],
        },
    })
}
