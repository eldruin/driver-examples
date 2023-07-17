//! Read the magnetometer measurement with the LSM303AGR sensor and transmit the
//! data through RTT.
//!
//! Install cargo-embed with:
//! `cargo install cargo-embed`
//!
//! Run with:
//! `cargo embed --example lsm303agr-mag-one-shot-mb`
//!
#![no_main]
#![no_std]

use cortex_m;
use cortex_m_rt::entry;
use lsm303agr::Lsm303agr;
use microbit::hal::i2c;
use microbit::hal::prelude::*;
use nb;
use panic_rtt_target as _;
use rtt_target::{rprintln, rtt_init_print};

#[entry]
fn main() -> ! {
    rtt_init_print!();
    rprintln!("LSM303AGR magnetometer example");
    if let Some(p) = microbit::Peripherals::take() {
        let gpio = p.GPIO.split();

        let scl = gpio.pin0.into_open_drain_input().into();
        let sda = gpio.pin30.into_open_drain_input().into();
        let i2c = i2c::I2c::i2c1(p.TWI1, sda, scl);

        let mut lsm = Lsm303agr::new_with_i2c(i2c);
        lsm.init().unwrap();
        loop {
            let data = nb::block!(lsm.magnetic_field()).unwrap();
            rprintln!("{:>4} {:>4} {:>4}", data.x_nt(), data.y_nt(), data.z_nt());

            for _ in 0..20_000 {
                cortex_m::asm::nop();
            }
        }
    }
    loop {
        continue;
    }
}
