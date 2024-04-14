//! Read the magnetometer measurement with the LSM303AGR sensor and transmit the
//! data through RTT.
//!
//! Install cargo-embed with:
//! `cargo install cargo-embed`
//!
//! Run with:
//! `cargo embed --example lsm303agr-mag-mb`
//!
#![no_main]
#![no_std]

use cortex_m_rt::entry;
use lsm303agr::{Lsm303agr, MagMode, MagOutputDataRate};
use microbit::hal::delay::Delay;
use microbit::hal::i2c;
use microbit::hal::prelude::*;
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
        let mut delay = Delay::new(p.TIMER0);

        let mut lsm = Lsm303agr::new_with_i2c(i2c);
        lsm.init().unwrap();
        lsm.set_mag_mode_and_odr(
            &mut delay,
            MagMode::HighResolution,
            MagOutputDataRate::Hz100,
        )
        .unwrap();
        let mut lsm = lsm.into_mag_continuous().ok().unwrap();
        loop {
            let status = lsm.mag_status().unwrap();
            if status.xyz_new_data() {
                let data = lsm.magnetic_field().unwrap();
                rprintln!("{:>4} {:>4} {:>4}", data.x_nt(), data.y_nt(), data.z_nt());
            }
            for _ in 0..20_000 {
                cortex_m::asm::nop();
            }
        }
    }
    loop {
        continue;
    }
}
