#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]

mod app_config;
mod handler;
mod resources;
mod tx_builder;
mod wifi;

use {defmt_rtt as _, panic_probe as _};

use core::ptr::addr_of_mut;
use embassy_executor::Spawner;
use embassy_net::dns::DnsSocket;
use embassy_net::tcp::client::{TcpClient, TcpClientState};
use embassy_time::Timer;
use embedded_alloc::LlffHeap as Heap;
use reqwless::client::HttpClient;

use crate::resources::{AssignedResources, ConfigPins, WiFiPins};
use libs::crypto::Crypto;
use libs::gas_station_client::{GasStationClient, ObjectID};

extern crate alloc;

#[global_allocator]
static HEAP: Heap = Heap::empty();

/*
pub fn get_sensor_unique_id(pin_flash: Peri<'static, FLASH>) -> u64 {
    let mut uid = [0u8; 8]; // 64-bit unique ID
    let mut flash = Flash::<_, Blocking, 2048>::new_blocking(pin_flash);
    flash
        .blocking_unique_id(&mut uid)
        .expect("Failed to get unique ID");
    u64::from_be_bytes(uid)
}
*/

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    {
        use core::mem::MaybeUninit;
        const HEAP_SIZE: usize = 1024 * 10;
        static mut HEAP_MEM: [MaybeUninit<u8>; HEAP_SIZE] = [MaybeUninit::uninit(); HEAP_SIZE];
        unsafe { HEAP.init(addr_of_mut!(HEAP_MEM) as usize, HEAP_SIZE) }
    }

    let config = app_config::load();

    let p = embassy_rp::init(Default::default());
    let pins: AssignedResources = split_resources!(p);

    // Using a test seed here, in the real world this should be abstracted with an HSM or a secure enclave.
    let mut kp = Crypto::from_seed([0; 32]);

    let stack = wifi::initialize_wifi(
        config.wifi.ssid.as_str(),
        config.wifi.pass.as_str(),
        _spawner,
        pins.wifi_pins,
    )
    .await;

    let client_state = TcpClientState::<4, 4096, 4096>::new();
    let tcp_client = TcpClient::new(stack, &client_state);
    let dns_client = DnsSocket::new(stack);

    let mut gas_client = GasStationClient::new(
        HttpClient::new(&tcp_client, &dns_client),
        config.gas_station.url.as_str(),
        config.gas_station.bearer.as_str(),
    );

    let package_id = ObjectID::from_hex(config.contract.package_id.as_str())
        .unwrap()
        .as_tx_object_id();

    loop {
        handler::run_handler(&mut kp, &mut gas_client, package_id).await;

        Timer::after_secs(30).await;
    }
}
