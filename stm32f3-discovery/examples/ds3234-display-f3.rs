//! Stores the date and time on a DS3234 real-time clock (RTC).
//! Then reads the date and time repeatedly and prints it to an
//! SSD1306 OLED display.
//!
//! This example is runs on the STM32F3 Discovery board using SPI1 and I2C1.
//!
//! ```
//! F3    <-> DS3234  <-> Display
//! GND   <-> GND     <-> GND
//! +3.3V <-> +3.3V
//! +5V               <-> +5V
//! PA5   <-> CLK
//! PA6   <-> DO
//! PA7   <-> DI
//! PB1   <-> CS
//! PB7               <-> SDA
//! PB6               <-> SCL
//! ```
//!
//! Run with:
//! `cargo run --example ds3234-display-f3`,

#![deny(unsafe_code)]
#![no_std]
#![no_main]

use core::convert::TryInto;
use core::fmt::Write;
use cortex_m_rt::entry;
use embedded_graphics::{
    mono_font::{ascii::FONT_6X10, MonoTextStyleBuilder},
    pixelcolor::BinaryColor,
    prelude::*,
    text::{Baseline, Text},
};
use embedded_hal::blocking::delay::DelayMs;
use embedded_hal::digital::v2::OutputPin;
use embedded_hal::spi::MODE_1;
use panic_rtt_target as _;
use rtt_target::{rprintln, rtt_init_print};
use ssd1306::{prelude::*, I2CDisplayInterface, Ssd1306};
use stm32f3xx_hal::{
    self as hal,
    delay::Delay,
    pac,
    prelude::*,
    spi::{config::Config, Spi},
};

use ds323x::{DateTimeAccess, Ds323x, NaiveDate};

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

    led.set_high().unwrap();
    delay.delay_ms(500_u16);
    led.set_low().unwrap();
    delay.delay_ms(500_u16);

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
    let interface = I2CDisplayInterface::new(i2c);
    let mut disp = Ssd1306::new(interface, DisplaySize128x64, DisplayRotation::Rotate0)
        .into_buffered_graphics_mode();
    disp.init().unwrap();
    disp.flush().unwrap();

    let text_style = MonoTextStyleBuilder::new()
        .font(&FONT_6X10)
        .text_color(BinaryColor::On)
        .build();

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
    let begin = NaiveDate::from_ymd(2020, 5, 2).and_hms(13, 50, 23);
    rtc.disable().unwrap();
    rtc.set_datetime(&begin).unwrap();
    rtc.enable().unwrap();
    loop {
        let now = rtc.datetime().unwrap();
        let mut line: heapless::String<32> = heapless::String::new();

        write!(line, "{}", now).unwrap();
        disp.clear();
        Text::with_baseline(&line, Point::zero(), text_style, Baseline::Top)
            .draw(&mut disp)
            .unwrap();
        disp.flush().unwrap();
    }
}
