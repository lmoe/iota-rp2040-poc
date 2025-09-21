use assign_resources::assign_resources;
use embassy_rp::Peri;
use embassy_rp::peripherals::{self};

assign_resources! {
    config: ConfigPins {
        flash: FLASH,
        bank: DMA_CH1,
    }

    wifi_pins: WiFiPins {
        pin_23: PIN_23,
        pin_24: PIN_24,
        pin_25: PIN_25,
        pin_29: PIN_29,
        dma_ch0: DMA_CH0,
        pio0: PIO0,
    }
}
