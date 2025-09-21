use crate::tx_builder;
use crate::tx_builder::{SensorReading, TemperatureReading};

use defmt::info;
use embedded_nal_async::{Dns, TcpConnect};
use libs::crypto::Crypto;
use libs::encoding::{Base64Signature, BcsData};
use libs::gas_station_client::GasStationClient;
use libs::transaction_types;

pub async fn run_handler<'a, TCP, DNS>(
    kp: &mut Crypto,
    gas_station_client: &mut GasStationClient<'a, TCP, DNS>,
    package_id: transaction_types::ObjectID,
) where
    TCP: TcpConnect + 'a,
    DNS: Dns + 'a,
{
    let gas_budget = 100000000;

    let reserved_gas = gas_station_client
        .reserve_gas(gas_budget, 40)
        .await
        .expect("Failed to reserve gas {}");

    if reserved_gas.gas_coins.is_empty() {
        info!("Gas coin is empty");
        return;
    }

    let tx = tx_builder::build_temperature_sensor_tx(
        reserved_gas.sponsor_address.as_tx_object_id(),
        reserved_gas.gas_coins[0].as_tx_object_ref(),
        kp.public_address().into(),
        package_id,
        gas_budget,
        SensorReading {
            location: heapless::String::try_from("ISS").unwrap(),
            sensor_id: 123,
            battery_reading: 255,
            data: TemperatureReading { temperature: 12345 },
        },
    );

    let tx_bytes = BcsData::new(tx);
    let signature = Base64Signature::new(&kp.sign(&tx_bytes.as_bcs_bytes().unwrap())).unwrap();

    let executed_tx = gas_station_client
        .execute_tx(reserved_gas.reservation_id, tx_bytes, signature)
        .await
        .expect("Failed to execute tx");

    info!("Success posting TX: {}", executed_tx.transaction_digest);
}
