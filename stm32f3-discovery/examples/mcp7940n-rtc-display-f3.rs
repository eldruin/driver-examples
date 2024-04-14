//! Stores the date and time on a MCP7940N real-time clock (RTC).
//! Then continuously print the date and time.
//!
//! Introductory blog post here:
//! https://blog.eldruin.com/mcp794xx-real-time-clock-rtc-driver-in-rust/
//!
//! This example is runs on the STM32F3 Discovery board using I2C1.
//!
//! ```
//! F3    <-> MCP7940N <-> Display
//! GND   <-> GND      <-> GND
//! +3.3V <-> +3.3V
//! +5V                <-> +5V
//! PB7   <-> SDA      <-> SDA
//! PB6   <-> SCL      <-> SCL
//! ```
//!
//! Beware that the MCP7940N runs on 3.3V but PB6 and PB7 run on 5V level
//! so make sure to put a logic level shifter in between.
//!
//! Run with:
//! `cargo run --example mcp7940n-rtc-display-f3 --target thumbv7em-none-eabihf`,

#![deny(unsafe_code)]
#![no_std]
#![no_main]

use core::fmt::Write;
use cortex_m_rt::entry;
use embedded_graphics::{
    mono_font::{ascii::FONT_6X10, MonoTextStyleBuilder},
    pixelcolor::BinaryColor,
    prelude::*,
    text::{Baseline, Text},
};
use panic_rtt_target as _;
use rtt_target::{rprintln, rtt_init_print};
use ssd1306::{prelude::*, I2CDisplayInterface, Ssd1306};
use stm32f3xx_hal::{self as hal, delay::Delay, pac, prelude::*};

use mcp794xx::{DateTimeAccess, Datelike, Mcp794xx, NaiveDate, Timelike};

#[entry]
fn main() -> ! {
    rtt_init_print!();
    rprintln!("MCP7940N example");

    let cp = cortex_m::Peripherals::take().unwrap();
    let dp = pac::Peripherals::take().unwrap();

    let mut flash = dp.FLASH.constrain();
    let mut rcc = dp.RCC.constrain();
    let mut gpioe = dp.GPIOE.split(&mut rcc.ahb);
    let clocks = rcc.cfgr.freeze(&mut flash.acr);

    let mut led = gpioe
        .pe9
        .into_push_pull_output(&mut gpioe.moder, &mut gpioe.otyper);
    let mut delay = Delay::new(cp.SYST, clocks);

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
        400.kHz().try_into().unwrap(),
        clocks,
        &mut rcc.apb1,
    );

    let manager = shared_bus::BusManagerSimple::new(i2c);
    let interface = I2CDisplayInterface::new(manager.acquire_i2c());
    let mut disp = Ssd1306::new(interface, DisplaySize128x64, DisplayRotation::Rotate0)
        .into_buffered_graphics_mode();
    disp.init().unwrap();
    disp.flush().unwrap();

    let text_style = MonoTextStyleBuilder::new()
        .font(&FONT_6X10)
        .text_color(BinaryColor::On)
        .build();

    let mut rtc = Mcp794xx::new_mcp7940n(manager.acquire_i2c());
    let begin = NaiveDate::from_ymd_opt(2022, 5, 2)
        .unwrap()
        .and_hms_opt(10, 21, 34)
        .unwrap();
    rtc.set_datetime(&begin).unwrap();
    rtc.enable().unwrap();
    loop {
        // Blink LED 0 to check that everything is actually running.
        // If the LED 0 is off, something went wrong.
        led.set_high().unwrap();
        delay.delay_ms(50_u8);
        led.set_low().unwrap();
        delay.delay_ms(50_u8);

        let now = rtc.datetime().unwrap();
        let mut buffer: heapless::String<32> = heapless::String::new();
        write!(
            buffer,
            "{}-{}-{} {}:{}:{} ",
            now.year(),
            now.month(),
            now.day(),
            now.hour(),
            now.minute(),
            now.second()
        )
        .unwrap();
        disp.clear();
        Text::with_baseline(&buffer, Point::zero(), text_style, Baseline::Top)
            .draw(&mut disp)
            .unwrap();
        disp.flush().unwrap();
    }
}
