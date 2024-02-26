use embedded_graphics::{
    image::{Image, ImageRaw},
    pixelcolor::BinaryColor,
    prelude::*,
};
use esp_idf_hal::{delay::FreeRtos, gpio::PinDriver};
use esp_idf_svc::{eventloop::EspSystemEventLoop, hal::peripherals::Peripherals};
mod display;
use display::Display;

fn main() -> anyhow::Result<()> {
    // It is necessary to call this function once. Otherwise some patches to the runtime
    // implemented by esp-idf-sys might not link properly. See https://github.com/esp-rs/esp-idf-template/issues/71
    esp_idf_svc::sys::link_patches();

    // Bind the log crate to the ESP Logging facilities
    esp_idf_svc::log::EspLogger::initialize_default();

    let peripherals = Peripherals::take().unwrap();
    let sysloop = EspSystemEventLoop::take()?;

    let Peripherals {
        modem, pins, i2c0, ..
    } = peripherals;

    let mut led = PinDriver::output(pins.gpio35)?;

    let rst = pins.gpio21;

    let sda = pins.gpio17;

    let scl = pins.gpio18;

    let mut display = Display::new(i2c0, rst, sda, scl)?;

    display.init()?;
    display.reset()?;

    log::info!("Hello, world!");

    display.print("Hello, world!").unwrap();

    let messages = vec!["Love Rust ❤️", "BCX2024 🚀", "Embedded Hardware 🛠"];

    let raw_image = ImageRaw::new(include_bytes!("./rust.raw"), 64);

    loop {
        led.set_high()?;
        FreeRtos::delay_ms(1000);

        display.show(raw_image.clone())?;

        led.set_low()?;
        FreeRtos::delay_ms(1000);

        for message in &messages {
            display.print(message).unwrap();
            FreeRtos::delay_ms(2000);
        }
    }
}
