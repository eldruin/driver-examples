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

use cortex_m_rt::entry;
use panic_rtt_target as _;
use rtt_target::{rprintln, rtt_init_print};
use stm32f3xx_hal::{self as hal, delay::Delay, pac, prelude::*};

use ds1307::{DateTimeAccess, Ds1307, NaiveDate};

#[entry]
fn main() -> ! {
    rtt_init_print!();
    rprintln!("DS1307 example");

    let cp = cortex_m::Peripherals::take().unwrap();
    let dp = pac::Peripherals::take().unwrap();

    let mut flash = dp.FLASH.constrain();
    let mut rcc = dp.RCC.constrain();
    let clocks = rcc.cfgr.freeze(&mut flash.acr);
    let mut delay = Delay::new(cp.SYST, clocks);

    let mut gpioe = dp.GPIOE.split(&mut rcc.ahb);
    let mut led = gpioe
        .pe9
        .into_push_pull_output(&mut gpioe.moder, &mut gpioe.otyper);

    let mut gpiob = dp.GPIOB.split(&mut rcc.ahb);
    let mut scl =
        gpiob
            .pb6
            .into_af4_open_drain(&mut gpiob.moder, &mut gpiob.otyper, &mut gpiob.afrl);
    let mut sda =
        gpiob
            .pb7
            .into_af4_open_drain(&mut gpiob.moder, &mut gpiob.otyper, &mut gpiob.afrl);
    scl.internal_pull_up(&mut gpiob.pupdr, true);
    sda.internal_pull_up(&mut gpiob.pupdr, true);

    let i2c = hal::i2c::I2c::new(
        dp.I2C1,
        (scl, sda),
        100.kHz().try_into().unwrap(),
        clocks,
        &mut rcc.apb1,
    );

    let mut rtc = Ds1307::new(i2c);
    let begin = NaiveDate::from_ymd_opt(2022, 5, 2)
        .unwrap()
        .and_hms_opt(10, 21, 34)
        .unwrap();
    rtc.set_datetime(&begin).unwrap();
    loop {
        let now = rtc.datetime().unwrap();
        if (now - begin).num_seconds() < 30 {
            // this will blink for 30 seconds
            led.set_high().unwrap();
            delay.delay_ms(250_u16);
            led.set_low().unwrap();
            delay.delay_ms(250_u16);
        }
    }
}
