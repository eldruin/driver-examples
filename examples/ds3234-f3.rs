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
//! PB7 <-> SDA
//! PB6 <-> SCL
//! ```
//!
//! Run with:
//! `cargo run --example ds3234-f3 --target thumbv7em-none-eabihf`,

#![no_std]
#![no_main]

// panic handler
extern crate panic_semihosting;

use cortex_m_rt::entry;
use embedded_hal::spi::MODE_0;
use f3::{
    hal::{delay::Delay, prelude::*, spi::Spi, stm32f30x},
    led::Led,
};

use ds323x::{DateTime, Ds323x, Hours};

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
        MODE_0,
        1.mhz(),
        clocks,
        &mut rcc.apb2,
    );

    let mut chip_select = gpiob
        .pb5
        .into_push_pull_output(&mut gpiob.moder, &mut gpiob.otyper);

    chip_select.set_high();

    let mut rtc = Ds323x::new_ds3234(spi, chip_select);
    let begin = DateTime {
        year: 2019,
        month: 1,
        day: 2,
        weekday: 3,
        hour: Hours::H24(4),
        minute: 5,
        second: 6,
    };
    rtc.set_datetime(&begin).unwrap();
    loop {
        let now = rtc.get_datetime().unwrap();
        if now.year == begin.year
            && now.month == begin.month
            && now.day == begin.day
            && now.weekday == begin.weekday
            && now.hour == begin.hour
            && now.minute == begin.minute
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
