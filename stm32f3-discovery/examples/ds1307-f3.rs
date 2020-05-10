//! Stores the date and time on a DS1307 real-time clock (RTC).
//! Then reads the date and time repeatedly and blinks LED 0
//! for the first 30 seconds.
//!
//! Introductory blog post here:
//! https://blog.eldruin.com/ds1307-real-time-clock-rtc-driver-in-rust/
//!
//! This example is runs on the STM32F3 Discovery board using I2C1.
//!
//! ```
//! F3  <-> DS1307
//! GND <-> GND
//! +5V <-> +5V
//! PB7 <-> SDA
//! PB6 <-> SCL
//! ```
//!
//! Run with:
//! `cargo run --example ds1307-f3 --target thumbv7em-none-eabihf`,

#![deny(unsafe_code)]
#![no_std]
#![no_main]

// panic handler
extern crate panic_semihosting;

use cortex_m_rt::entry;
use f3::{
    hal::{delay::Delay, i2c::I2c, prelude::*, stm32f30x},
    led::Led,
};

use ds1307::{Ds1307, NaiveDate, Rtcc};

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

    let mut rtc = Ds1307::new(i2c);
    let begin = NaiveDate::from_ymd(2020, 5, 2).and_hms(10, 21, 34);
    rtc.set_datetime(&begin).unwrap();
    loop {
        let now = rtc.get_datetime().unwrap();
        if (now - begin).num_seconds() < 30 {
            // this will blink for 30 seconds
            led.on();
            delay.delay_ms(250_u16);
            led.off();
            delay.delay_ms(250_u16);
        }
    }
}
