//! Continuously read the acceleration with a KXCJ9-1018 and
//! transmit it per USART. (trivially adaptable to use an KXCJ9-1008).
//!
//! When running you should be able to see the acceleration readings in your
//! serial communication program.
//!
//! Introductory blog post here:
//! https://blog.eldruin.com/kxcj9-kxcjb-tri-axis-mems-accelerator-driver-in-rust/
//!
//! This example is runs on the STM32F3 Discovery board using I2C1 and USART1.
//!
//! To setup the serial communication, have a look at the discovery book:
//! https://rust-embedded.github.io/discovery/10-serial-communication/index.html
//!
//! ```
//! F3    <-> KXCJ9
//! GND   <-> GND
//! +3.3V <-> VCC
//! PB7   <-> SDA
//! PB6   <-> SCL
//!
//! F3   <-> Serial device
//! GND  <-> GND
//! PA9  <-> TX
//! PA10 <-> RX
//! ```
//!
//! Beware that the KXCJ9 runs on 3.3V but PB6 and PB7 run on 5V level
//! so make sure to put a logic level shifter in between.
//!
//! Run with:
//! `cargo run --example kxcj9-usart-f3 --target thumbv7em-none-eabihf`,

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
};

use kxcj9::{Kxcj9, SlaveAddr};

use core::fmt::Write;
#[entry]
fn main() -> ! {
    let dp = stm32f30x::Peripherals::take().unwrap();

    let mut flash = dp.FLASH.constrain();
    let mut rcc = dp.RCC.constrain();
    let clocks = rcc.cfgr.freeze(&mut flash.acr);

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

    let mut accelerometer = Kxcj9::new_kxcj9_1018(i2c, SlaveAddr::default());
    accelerometer.enable().unwrap();

    loop {
        let accel = accelerometer.read().unwrap();

        // transform numbers to string
        let mut buffer: heapless::String<heapless::consts::U16> = heapless::String::new();
        write!(buffer, "{},{},{} ", accel.x, accel.y, accel.z).unwrap();

        // send buffer
        for byte in buffer.into_bytes().iter() {
            while usart1.isr.read().txe().bit_is_clear() {}
            usart1.tdr.write(|w| w.tdr().bits(u16::from(*byte)));
        }
    }
}
