//! Stores some data on an AT24C256C EEPROM.
//! Then reads it again and if it matches, blinks LED 0.
//!
//! Introductory blog post here:
//! https://blog.eldruin.com/24x-serial-eeprom-driver-in-rust/
//!
//! This example is runs on the STM32F3 Discovery board using I2C1.
//!
//! ```
//! F3  <-> AT24C256
//! GND <-> GND
//! +5V <-> +5V
//! PB7 <-> SDA
//! PB6 <-> SCL
//! ```
//!
//! Run with:
//! `cargo run --example at24c256-f3 --target thumbv7em-none-eabihf`,

#![deny(unsafe_code)]
#![no_std]
#![no_main]

use panic_semihosting as _;

use cortex_m_rt::entry;
use f3::{
    hal::{delay::Delay, i2c::I2c, prelude::*, stm32f30x},
    led::Led,
};

use eeprom24x::{Eeprom24x, SlaveAddr};

#[entry]
fn main() -> ! {
    let cp = cortex_m::Peripherals::take().unwrap();
    let dp = stm32f30x::Peripherals::take().unwrap();

    let mut flash = dp.FLASH.constrain();
    let mut rcc = dp.RCC.constrain();
    let clocks = rcc.cfgr.freeze(&mut flash.acr);
    let mut delay = Delay::new(cp.SYST, clocks);

    let mut gpioe = dp.GPIOE.split(&mut rcc.ahb);
    let mut led: Led = gpioe
        .pe9
        .into_push_pull_output(&mut gpioe.moder, &mut gpioe.otyper)
        .into();

    let mut gpiob = dp.GPIOB.split(&mut rcc.ahb);
    let scl = gpiob.pb6.into_af4(&mut gpiob.moder, &mut gpiob.afrl);
    let sda = gpiob.pb7.into_af4(&mut gpiob.moder, &mut gpiob.afrl);

    let i2c = I2c::i2c1(dp.I2C1, (scl, sda), 100.khz(), clocks, &mut rcc.apb1);

    let mut eeprom = Eeprom24x::new_24x256(i2c, SlaveAddr::Alternative(true, true, true));
    let memory_address = 0x01;
    eeprom
        .write_page(memory_address, &[0xAB, 0xCD, 0xEF, 0x12])
        .unwrap();

    // wait maximum time necessary for write
    delay.delay_ms(5_u16);
    loop {
        let mut data = [0; 4];
        eeprom.read_data(memory_address, &mut data).unwrap();
        if data == [0xAB, 0xCD, 0xEF, 0x12] {
            led.on();
            delay.delay_ms(500_u16);
            led.off();
            delay.delay_ms(500_u16);
        }
    }
}
