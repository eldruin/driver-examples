//! Measure the eCO2 and eTVOC in the air, compensate for the ambient temperature
//! and humidity with an HDC2080 sensor and print the data in CSV format every
//! 10 seconds.
//!
//! ```
//! RPi   <-> CCS811 <-> HDC2080
//! GND   <-> GND    <-> GND
//! 3.3V  <-> VCC    <-> VCC
//! Pin 5 <-> SCL    <-> SCL
//! Pin 3 <-> SDA    <-> SDA
//! GND   <-> nWAKE
//! 3.3V  <-> RST
//! ```
//!
//! Run with:
//! `cargo run --example ccs811-gas-voc-logging-rpi`
//!

use embedded_ccs811::{
    prelude::*, Ccs811Awake, MeasurementMode, ModeChangeError, SlaveAddr as Ccs811Addr,
};
use embedded_hal::blocking::delay::DelayMs;
use hdc20xx::{Hdc20xx, SlaveAddr as Hdc20xxAddr};
use linux_embedded_hal::{Delay, I2cdev};
use nb::block;

fn main() {
    let dev = I2cdev::new("/dev/i2c-1").unwrap();
    let bus = shared_bus::BusManagerStd::new(dev);
    let mut delay = Delay {};
    let mut hdc2080 = Hdc20xx::new(bus.acquire_i2c(), Hdc20xxAddr::default());
    let ccs811 = Ccs811Awake::new(bus.acquire_i2c(), Ccs811Addr::default());
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
            println!("eco2,etvoc,raw_current,raw_voltage,temperature,humidity");
            loop {
                let data = block!(ccs811.data()).unwrap();
                println!(
                    "{},{},{},{},{:.2},{:.2}",
                    data.eco2,
                    data.etvoc,
                    data.raw_current,
                    data.raw_voltage,
                    env.temperature,
                    env.humidity.unwrap_or(0.0)
                );
                env = block!(hdc2080.read()).unwrap();
                ccs811
                    .set_environment(env.temperature, env.humidity.unwrap_or(0.0))
                    .unwrap();
                delay.delay_ms(10_000_u32); // wait 10 seconds
            }
        }
    }
}
