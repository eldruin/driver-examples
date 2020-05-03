//! Stores the date and time on a DS3234 real-time clock (RTC).
//! Then reads the date and time repeatedly and if everything but the
//! seconds match, blinks LED 0.
//! After 1 minute it will stop blinking as the minutes will not match
//! anymore.
//!
//! This example is runs on the STM32F3 Discovery board using SPI1.
//!
//! ```
//! F3  <-> DS3234
//! GND <-> GND
//! +5V <-> +5V
//! PA5 <-> CLK
//! PA6 <-> DO
//! PA7 <-> DI
//! PB1 <-> CS
//! ```
//!
//! Run with:
//! `cargo run --example ds3234-f3 --target thumbv7em-none-eabihf`,

#![deny(unsafe_code)]
#![no_std]
#![no_main]

// panic handler
extern crate panic_semihosting;

use cortex_m_rt::entry;
use embedded_hal::blocking::delay::DelayMs;
use embedded_hal::digital::v2::OutputPin;
use embedded_hal::spi::MODE_1;
use f3::{
    hal::{
        delay::Delay, flash::FlashExt, gpio::GpioExt, rcc::RccExt, spi::Spi, stm32f30x,
        time::U32Ext,
    },
    led::Led,
};

use chrono::{Datelike, Timelike};
use ds323x::{Ds323x, NaiveDate};
use rtcc::Rtcc;

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

    let mut gpioa = dp.GPIOA.split(&mut rcc.ahb);
    let mut gpiob = dp.GPIOB.split(&mut rcc.ahb);

    // SPI configuration
    let sck = gpioa.pa5.into_af5(&mut gpioa.moder, &mut gpioa.afrl);
    let miso = gpioa.pa6.into_af5(&mut gpioa.moder, &mut gpioa.afrl);
    let mosi = gpioa.pa7.into_af5(&mut gpioa.moder, &mut gpioa.afrl);

    let spi = Spi::spi1(
        dp.SPI1,
        (sck, miso, mosi),
        MODE_1,
        1.mhz(),
        clocks,
        &mut rcc.apb2,
    );

    let mut chip_select = gpiob
        .pb1
        .into_push_pull_output(&mut gpiob.moder, &mut gpiob.otyper);

    chip_select.set_high().unwrap();

    let mut rtc = Ds323x::new_ds3234(spi, chip_select);
    let begin = NaiveDate::from_ymd(2020, 5, 2).and_hms(13, 50, 23);
    rtc.set_datetime(&begin).unwrap();
    loop {
        let now = rtc.get_datetime().unwrap();
        if now.year() == begin.year()
            && now.month() == begin.month()
            && now.day() == begin.day()
            && now.hour() == begin.hour()
            && now.minute() == begin.minute()
        {
            // as we do not compare the seconds, this will blink for one
            // minute and then stop.
            led.on();
            delay.delay_ms(500_u16);
            led.off();
            delay.delay_ms(500_u16);
        }
    }
}
