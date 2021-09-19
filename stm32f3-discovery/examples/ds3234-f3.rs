//! Stores the date and time on a DS3234 real-time clock (RTC).
//! Then reads the date and time repeatedly and blink LED 0
//! for 30 seconds.
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

use cortex_m_rt::entry;
use embedded_hal::blocking::delay::DelayMs;
use embedded_hal::digital::v2::OutputPin;
use embedded_hal::spi::MODE_1;
use panic_rtt_target as _;
use rtt_target::{rprintln, rtt_init_print};
use stm32f3xx_hal::{
    delay::Delay,
    pac,
    prelude::*,
    spi::{config::Config, Spi},
};

use ds323x::{Ds323x, NaiveDate, Rtcc};

#[entry]
fn main() -> ! {
    rtt_init_print!();
    rprintln!("DS3234 example");

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

    let mut gpioa = dp.GPIOA.split(&mut rcc.ahb);
    let mut gpiob = dp.GPIOB.split(&mut rcc.ahb);

    // SPI configuration
    let sck = gpioa
        .pa5
        .into_af5_push_pull(&mut gpioa.moder, &mut gpioa.otyper, &mut gpioa.afrl);
    let miso = gpioa
        .pa6
        .into_af5_push_pull(&mut gpioa.moder, &mut gpioa.otyper, &mut gpioa.afrl);
    let mosi = gpioa
        .pa7
        .into_af5_push_pull(&mut gpioa.moder, &mut gpioa.otyper, &mut gpioa.afrl);

    let spi_config = Config::default().frequency(1.MHz()).mode(MODE_1);
    let spi = Spi::new(
        dp.SPI1,
        (sck, miso, mosi),
        spi_config,
        clocks,
        &mut rcc.apb2,
    );

    let mut chip_select = gpiob
        .pb1
        .into_push_pull_output(&mut gpiob.moder, &mut gpiob.otyper);

    chip_select.set_high().unwrap();

    let mut rtc = Ds323x::new_ds3234(spi, chip_select);
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
