//! Measure the acceleration with the LSM303AGR sensor and transmit the
//! data through the serial interface.
//!
//! Install cargo-embed with:
//! `cargo install cargo-embed`
//!
//! Run with:
//! `cargo embed --example lsm303agr-accel-mb`
//!
#![no_main]
#![no_std]

use core::fmt::Write;
use cortex_m_rt::entry;
use lsm303agr::{AccelMode, AccelOutputDataRate, Lsm303agr};
use microbit::hal::delay::Delay;
use microbit::hal::i2c;
use microbit::hal::prelude::*;
use microbit::hal::serial;
use microbit::hal::serial::BAUD115200;
use panic_rtt_target as _;
use rtt_target::{rprintln, rtt_init_print};

#[entry]
fn main() -> ! {
    rtt_init_print!();
    rprintln!("LSM303AGR accelerometer example");
    if let Some(p) = microbit::Peripherals::take() {
        let gpio = p.GPIO.split();

        let tx = gpio.pin24.into_push_pull_output().into();
        let rx = gpio.pin25.into_floating_input().into();

        let (mut tx, _) = serial::Serial::uart0(p.UART0, tx, rx, BAUD115200).split();

        let _ = write!(&mut tx, "\n\rAccelerometer\n\r");

        let scl = gpio.pin0.into_open_drain_input().into();
        let sda = gpio.pin30.into_open_drain_input().into();
        let i2c = i2c::I2c::i2c1(p.TWI1, sda, scl);
        let mut delay = Delay::new(p.TIMER0);

        let mut accel = Lsm303agr::new_with_i2c(i2c);
        accel.init().unwrap();
        accel
            .set_accel_mode_and_odr(&mut delay, AccelMode::Normal, AccelOutputDataRate::Hz10)
            .unwrap();
        loop {
            let status = accel.accel_status().unwrap();
            if status.xyz_new_data() {
                let data = accel.acceleration().unwrap();
                let _ = write!(
                    &mut tx,
                    "{:>4} {:>4} {:>4}\n\r",
                    data.x_mg(),
                    data.y_mg(),
                    data.z_mg()
                );
            }
            for _ in 0..200_000 {
                cortex_m::asm::nop();
            }
        }
    }
    loop {
        continue;
    }
}
