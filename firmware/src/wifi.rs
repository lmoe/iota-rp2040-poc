use crate::resources::WiFiPins;
use cyw43::JoinOptions;
use cyw43_pio::{DEFAULT_CLOCK_DIVIDER, PioSpi};
use defmt::{info, unwrap};
use embassy_executor::Spawner;
use embassy_net::{Ipv4Address, Ipv4Cidr, Stack, StackResources};
use embassy_rp::bind_interrupts;
use embassy_rp::clocks::RoscRng;
use embassy_rp::gpio::{Level, Output};
use embassy_rp::peripherals::{DMA_CH0, PIO0};
use embassy_rp::pio::{InterruptHandler, Pio};
use static_cell::StaticCell;
use {defmt_rtt as _, panic_probe as _};

bind_interrupts!(struct Irqs {
    PIO0_IRQ_0 => InterruptHandler<PIO0>;
});

#[embassy_executor::task]
async fn cyw43_task(
    runner: cyw43::Runner<'static, Output<'static>, PioSpi<'static, PIO0, 0, DMA_CH0>>,
) -> ! {
    runner.run().await
}

#[embassy_executor::task]
async fn net_task(mut runner: embassy_net::Runner<'static, cyw43::NetDriver<'static>>) -> ! {
    runner.run().await
}

// We can either include the wifi driver directly into the firmware with each build,
// or we can upload the driver once in a certain location and only reference the location un boot.
// The latter makes building and uploading the firmware faster, but if the firmware size grows into the wifi driver location
// Things go boom, without proper error reception.

#[cfg(feature = "wifi_driver")]
fn get_firmware() -> (&'static [u8; 230321], &'static [u8; 4752]) {
    let fw = include_bytes!("../../cyw43/43439A0.bin");
    let clm = include_bytes!("../../cyw43/43439A0_clm.bin");
    (fw, clm)
}

#[cfg(not(feature = "wifi_driver"))]
fn get_firmware() -> (&'static [u8], &'static [u8]) {
    // probe-rs download 43439A0.bin --binary-format bin --chip RP2040 --base-address 0x10100000
    // probe-rs download 43439A0_clm.bin --binary-format bin --chip RP2040 --base-address 0x10140000

    let fw = unsafe { core::slice::from_raw_parts(0x10100000 as *const u8, 230321) };
    let clm = unsafe { core::slice::from_raw_parts(0x10140000 as *const u8, 4752) };
    (fw, clm)
}

pub async fn initialize_wifi<'a>(
    wifi_name: &str,
    wifi_password: &str,
    spawner: Spawner,
    pins: WiFiPins,
) -> Stack<'static> {
    let mut rng = RoscRng;

    let (fw, clm) = get_firmware();

    let pwr = Output::new(pins.pin_23, Level::Low);
    let cs = Output::new(pins.pin_25, Level::High);
    let mut pio = Pio::new(pins.pio0, Irqs);
    let spi = PioSpi::new(
        &mut pio.common,
        pio.sm0,
        DEFAULT_CLOCK_DIVIDER,
        pio.irq0,
        cs,
        pins.pin_24,
        pins.pin_29,
        pins.dma_ch0,
    );

    static STATE: StaticCell<cyw43::State> = StaticCell::new();
    let state = STATE.init(cyw43::State::new());
    let (net_device, mut control, runner) = cyw43::new(state, pwr, spi, fw).await;
    unwrap!(spawner.spawn(cyw43_task(runner)));

    control.init(clm).await;
    control
        .set_power_management(cyw43::PowerManagementMode::PowerSave)
        .await;

    // Too lazy to make DHCP/Static configurable as StaticConfigV4 does not implement serde::Deserialize
    // Using static here, as connection establishes much faster.

    //let config = Config::dhcpv4(Default::default());
    let config = embassy_net::Config::ipv4_static(embassy_net::StaticConfigV4 {
        address: Ipv4Cidr::new(Ipv4Address::new(192, 168, 178, 98), 24),
        gateway: Some(Ipv4Address::new(192, 168, 178, 1)),
        dns_servers: Default::default(),
    });

    let seed = rng.next_u64();

    static RESOURCES: StaticCell<StackResources<5>> = StaticCell::new();
    let (stack, runner) = embassy_net::new(
        net_device,
        config,
        RESOURCES.init(StackResources::new()),
        seed,
    );

    unwrap!(spawner.spawn(net_task(runner)));

    while let Err(err) = control
        .join(wifi_name, JoinOptions::new(wifi_password.as_bytes()))
        .await
    {
        info!("join failed with status={}", err.status);
    }

    info!("waiting for link...");
    stack.wait_link_up().await;

    info!("waiting for connectivity");
    stack.wait_config_up().await;

    info!("Stack is up!");

    stack
}
