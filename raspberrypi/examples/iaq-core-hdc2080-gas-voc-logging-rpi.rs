//! Measure the CO2 and TVOC in the air and the ambient temperature
//! and humidity with an HDC2080 sensor and print the data in CSV
//! format every 10 seconds.
//!
//!
//! ```
//! RPi   <-> iAQ-Core <-> HDC2080
//! GND   <-> GND      <-> GND
//! 3.3V  <-> VCC      <-> VCC
//! Pin 5 <-> SCL      <-> SCL
//! Pin 3 <-> SDA      <-> SDA
//! GND                <-> A0
//! ```
//!
//! Run with:
//! `cargo run --example iaq-core-hdc2080-gas-voc-logging-rpi`
//!

use embedded_hal::blocking::delay::DelayMs;
use hdc20xx::{Hdc20xx, SlaveAddr as Hdc20xxAddr};
use iaq_core::IaqCore;
use linux_embedded_hal::{Delay, I2cdev};
use nb::block;

fn main() {
    let dev = I2cdev::new("/dev/i2c-1").unwrap();
    let bus = shared_bus::BusManagerStd::new(dev);
    let mut delay = Delay {};
    let mut hdc2080 = Hdc20xx::new(bus.acquire_i2c(), Hdc20xxAddr::default());
    let mut iaq_core = IaqCore::new(bus.acquire_i2c());

    println!("co2,tvoc,resistance,temperature,humidity");
    loop {
        let data = block!(iaq_core.data()).unwrap();
        let env = block!(hdc2080.read()).unwrap();
        println!(
            "{},{},{},{:.2},{:.2}",
            data.co2,
            data.tvoc,
            data.resistance,
            env.temperature,
            env.humidity.unwrap_or(0.0)
        );
        delay.delay_ms(10_000_u32); // wait 10 seconds
    }
}
