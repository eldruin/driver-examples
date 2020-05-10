//! Stores the date and time on a DS1307 real-time clock (RTC).
//! Then reads the date and time repeatedly blink LED 0 for 30 seconds.
//!
//! Introductory blog post here:
//! https://blog.eldruin.com/ds1307-real-time-clock-rtc-driver-in-rust/
//!
//! This example is runs on the STM32F1 "BluePill" board using I2C1.
//!
//! ```
//! BP  <-> DS1307
//! GND <-> GND
//! +5V <-> +5V
//! PB8 <-> SCL
//! PB9 <-> SDA
//! ```
//!
//! Run with:
//! `cargo run --example ds1307-rtc-bp`,

#![deny(unsafe_code)]
#![no_std]
#![no_main]

use cortex_m_rt::entry;
use ds1307::{Ds1307, NaiveDate, Rtcc};
use embedded_hal::digital::v2::OutputPin;
use panic_semihosting as _;
use stm32f1xx_hal::{
    delay::Delay,
    i2c::{BlockingI2c, DutyCycle, Mode},
    pac,
    prelude::*,
};

#[entry]
fn main() -> ! {
    let cp = cortex_m::Peripherals::take().unwrap();
    let dp = pac::Peripherals::take().unwrap();

    let mut flash = dp.FLASH.constrain();
    let mut rcc = dp.RCC.constrain();

    let clocks = rcc.cfgr.freeze(&mut flash.acr);

    let mut afio = dp.AFIO.constrain(&mut rcc.apb2);

    let mut gpiob = dp.GPIOB.split(&mut rcc.apb2);

    let scl = gpiob.pb8.into_alternate_open_drain(&mut gpiob.crh);
    let sda = gpiob.pb9.into_alternate_open_drain(&mut gpiob.crh);

    let i2c = BlockingI2c::i2c1(
        dp.I2C1,
        (scl, sda),
        &mut afio.mapr,
        Mode::Fast {
            frequency: 100_000.hz(),
            duty_cycle: DutyCycle::Ratio2to1,
        },
        clocks,
        &mut rcc.apb1,
        1000,
        10,
        1000,
        1000,
    );

    let mut gpioc = dp.GPIOC.split(&mut rcc.apb2);
    let mut led = gpioc.pc13.into_push_pull_output(&mut gpioc.crh);
    let mut delay = Delay::new(cp.SYST, clocks);

    let mut rtc = Ds1307::new(i2c);
    let begin = NaiveDate::from_ymd(2020, 5, 2).and_hms(10, 21, 34);
    rtc.set_datetime(&begin).unwrap();
    loop {
        let now = rtc.get_datetime().unwrap();
        if (now - begin).num_seconds() < 30 {
            // this will blink for 30 seconds
            led.set_high().unwrap();
            delay.delay_ms(250_u16);
            led.set_low().unwrap();
            delay.delay_ms(250_u16);
        }
    }
}
