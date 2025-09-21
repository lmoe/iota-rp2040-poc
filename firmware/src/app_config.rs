use embassy_net::StaticConfigV4;

#[derive(serde::Deserialize)]
pub struct ContractConfig {
    pub package_id: heapless::String<64>,
}

#[derive(serde::Deserialize)]
pub struct WifiConfig {
    pub ssid: heapless::String<32>,
    pub pass: heapless::String<63>,
}

#[derive(serde::Deserialize)]
pub struct GasStationConfig {
    pub url: heapless::String<256>,
    pub bearer: heapless::String<256>,
}

#[derive(serde::Deserialize)]
pub struct AppConfig {
    pub contract: ContractConfig,
    pub wifi: WifiConfig,
    pub gas_station: GasStationConfig,
}

pub fn load() -> AppConfig {
    const MERGED: &str = include_str!(concat!(env!("OUT_DIR"), "/config.json"));

    let (cfg, _consumed): (AppConfig, usize) =
        serde_json_core::from_str(MERGED).expect("parse config");

    cfg
}
