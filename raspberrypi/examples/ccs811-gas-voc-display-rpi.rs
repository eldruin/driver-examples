//! Continuously measure the eCO2 and eTVOC in the air and print it to an
//! SSD1306 OLED display.
//! In order to compensate for the ambient temperature and humidity, an HDC2080
//! sensor is used.
//!
//! Introductory blog post with some pictures here:
//! https://blog.eldruin.com/ccs811-indoor-air-quality-sensor-driver-in-rust/
//!
//! ```
//! RPi   <-> CCS811 <-> HDC2080 <-> Display
//! GND   <-> GND    <-> GND     <-> GND
//! 3.3V  <-> VCC    <-> VCC     <-> VDD
//! Pin 5 <-> SCL    <-> SCL     <-> SCL
//! Pin 3 <-> SDA    <-> SDA     <-> SDA
//! GND   <-> nWAKE
//! 3.3V  <-> RST
//! ```
//!
//! Run with:
//! `cargo run --example ccs811-gas-voc-display-rpi`,
//!
use core::fmt::Write;
use embedded_ccs811::{
    prelude::*, Ccs811Awake, MeasurementMode, ModeChangeError, SlaveAddr as Ccs811Addr,
};
use embedded_graphics::{
    mono_font::{ascii::FONT_6X10, MonoTextStyleBuilder},
    pixelcolor::BinaryColor,
    prelude::*,
    text::{Baseline, Text},
};
use embedded_hal::blocking::delay::DelayMs;
use hdc20xx::{Hdc20xx, SlaveAddr as Hdc20xxAddr};
use linux_embedded_hal::{Delay, I2cdev};
use nb::block;
use ssd1306::{prelude::*, I2CDisplayInterface, Ssd1306};

fn main() {
    let dev = I2cdev::new("/dev/i2c-1").unwrap();
    let bus = shared_bus::BusManagerStd::new(dev);
    let mut delay = Delay {};
    let interface = I2CDisplayInterface::new(bus.acquire_i2c());
    let mut disp = Ssd1306::new(interface, DisplaySize128x64, DisplayRotation::Rotate0)
        .into_buffered_graphics_mode();
    disp.init().unwrap();
    disp.flush().unwrap();

    let text_style = MonoTextStyleBuilder::new()
        .font(&FONT_6X10)
        .text_color(BinaryColor::On)
        .build();

    let mut hdc2080 = Hdc20xx::new(bus.acquire_i2c(), Hdc20xxAddr::default());
    let mut ccs811 = Ccs811Awake::new(bus.acquire_i2c(), Ccs811Addr::default());
    ccs811.software_reset().unwrap();
    delay.delay_ms(10_u16);

    match ccs811.start_application() {
        Err(ModeChangeError { dev: _, error }) => {
            println!("Error during application start: {:?}", error);
        }
        Ok(mut ccs811) => {
            let mut env = block!(hdc2080.read()).unwrap();
            ccs811
                .set_environment(env.temperature, env.humidity.unwrap_or(0.0))
                .unwrap();
            ccs811.set_mode(MeasurementMode::ConstantPower1s).unwrap();
            loop {
                let mut lines = [String::new(), String::new(), String::new(), String::new()];
                let data = block!(ccs811.data()).unwrap();
                write!(lines[0], "eCO2: {}", data.eco2).unwrap();
                write!(lines[1], "eTVOC: {}", data.etvoc).unwrap();
                write!(lines[2], "Temp: {:.2}ºC", env.temperature).unwrap();
                write!(lines[3], "Humidity: {:.2}%", env.humidity.unwrap_or(0.0)).unwrap();
                disp.clear();
                for (i, line) in lines.iter().enumerate() {
                    Text::with_baseline(
                        line,
                        Point::new(0, i as i32 * 16),
                        text_style,
                        Baseline::Top,
                    )
                    .draw(&mut disp)
                    .unwrap();
                }
                disp.flush().unwrap();

                env = block!(hdc2080.read()).unwrap();
                ccs811
                    .set_environment(env.temperature, env.humidity.unwrap_or(0.0))
                    .unwrap();
                delay.delay_ms(10_000_u32); // wait 10 seconds
            }
        }
    }
}
