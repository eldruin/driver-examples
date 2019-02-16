//! Continuously read the temperature every second with the LM75 and
//! transmit it per USART.
//!
//! When running you should be able to see the temperature readings in your
//! serial communication program.
//!
//! This example is runs on the STM32F3 Discovery board using I2C1 and USART1.
//!
//! To setup the serial communication, have a look at the discovery book:
//! https://rust-embedded.github.io/discovery/10-serial-communication/index.html
//!
//! ```
//! F3  <-> LM75
//! GND <-> GND
//! VCC <-> +5V
//! PB7 <-> SDA
//! PB6 <-> SCL
//!
//! F3   <-> Serial device
//! GND  <-> GND
//! PA9  <-> TX
//! PA10 <-> RX
//! ```
//!
//! Run with:
//! `cargo run --example f3-usart --target thumbv7em-none-eabihf`,

#![no_std]
#![no_main]

// panic handler
extern crate panic_semihosting;

pub use cortex_m::{asm::bkpt, iprint, iprintln, peripheral::ITM};
use cortex_m_rt::entry;
use f3::hal::{
    i2c::I2c,
    prelude::*,
    serial::Serial,
    stm32f30x::{self, USART1},
    timer::Timer,
};

use lm75::{Lm75, SlaveAddr};
use nb::block;

use core::fmt::Write;
#[entry]
fn main() -> ! {
    let dp = stm32f30x::Peripherals::take().unwrap();

    let mut flash = dp.FLASH.constrain();
    let mut rcc = dp.RCC.constrain();
    let clocks = rcc.cfgr.freeze(&mut flash.acr);

    let mut timer = Timer::tim2(dp.TIM2, 1.hz(), clocks, &mut rcc.apb1);

    let mut gpioa = dp.GPIOA.split(&mut rcc.ahb);
    let tx = gpioa.pa9.into_af7(&mut gpioa.moder, &mut gpioa.afrh);
    let rx = gpioa.pa10.into_af7(&mut gpioa.moder, &mut gpioa.afrh);

    Serial::usart1(dp.USART1, (tx, rx), 115_200.bps(), clocks, &mut rcc.apb2);
    let usart1: &'static stm32f30x::usart1::RegisterBlock =
        unsafe { &mut *(USART1::ptr() as *mut _) };

    let mut gpiob = dp.GPIOB.split(&mut rcc.ahb);
    let scl = gpiob.pb6.into_af4(&mut gpiob.moder, &mut gpiob.afrl);
    let sda = gpiob.pb7.into_af4(&mut gpiob.moder, &mut gpiob.afrl);

    let i2c = I2c::i2c1(dp.I2C1, (scl, sda), 100.khz(), clocks, &mut rcc.apb1);

    let mut lm75 = Lm75::new(i2c, SlaveAddr::Alternative(false, false, false));

    loop {
        block!(timer.wait()).unwrap();

        let temp = lm75.read_temperature().unwrap();

        // transform number to string
        let mut buffer: heapless::String<heapless::consts::U16> = heapless::String::new();
        write!(buffer, "{} ", temp).unwrap();

        // send buffer
        for byte in buffer.into_bytes().iter() {
            while usart1.isr.read().txe().bit_is_clear() {}
            usart1.tdr.write(|w| w.tdr().bits(u16::from(*byte)));
        }
    }
}
