//! Continuously measure the ambient light color using
//! two VEML6040 which share the same address through a TCA9548A
//! and print it to an SSD1306 OLED display.
//!
//! This example is runs on the STM32F3 Discovery board using I2C1.
//!
//! ```
//! F3   <-> TCA9548A <-> Display <-> VEML6040 <-> VEML6040
//! GND  <-> GND      <-> GND     <-> GND      <-> GND
//! 3.3V <-> VCC      <-> VDD     <-> VCC      <-> VCC
//! PB7  <-> SDA      <-> SDA
//! PB6  <-> SCL      <-> SCL
//!          SDA0                 <-> SDA
//!          SCL0                 <-> SCL
//!          SDA1                              <-> SDA
//!          SCL1                              <-> SCL
//! ```
//!
//! Run with:
//! `cargo run --example tca9548a-multiple-veml6040-display-f3 --target thumbv7em-none-eabihf`,

#![deny(unsafe_code)]
#![no_std]
#![no_main]

use core::convert::TryInto;
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

use veml6040::Veml6040;
use xca9548a::{SlaveAddr, Xca9548a};

use core::fmt::Write;
#[entry]
fn main() -> ! {
    rtt_init_print!();
    rprintln!("TCA9548A example");

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
        100.kHz().try_into().unwrap(),
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

    let i2c_switch = Xca9548a::new(manager.acquire_i2c(), SlaveAddr::default());
    let parts = i2c_switch.split();
    let mut sensor0 = Veml6040::new(parts.i2c0);
    let mut sensor1 = Veml6040::new(parts.i2c1);
    sensor0.enable().unwrap();
    sensor1.enable().unwrap();
    loop {
        // Blink LED 0 to check that everything is actually running.
        // If the LED 0 is off, something went wrong.
        led.set_high().unwrap();
        delay.delay_ms(50_u16);
        led.set_low().unwrap();
        delay.delay_ms(50_u16);

        let mut lines: [heapless::String<32>; 2] =
            [heapless::String::new(), heapless::String::new()];

        let m0 = sensor0.read_all_channels().unwrap();
        let m1 = sensor1.read_all_channels().unwrap();

        write!(
            lines[0],
            "Sensor 0: R {} G {} B {} W {}",
            m0.red, m0.green, m0.blue, m0.white
        )
        .unwrap();
        write!(
            lines[1],
            "Sensor 1: R {} G {} B {} W {}     ",
            m1.red, m1.green, m1.blue, m1.white
        )
        .unwrap();

        disp.clear();
        for (i, line) in lines.iter().enumerate() {
            Text::with_baseline(
                line,
                Point::new(0, i as i32 * 16),
                text_style,
                Baseline::Top,
            )
            .draw(&mut disp)
            .unwrap();
        }
        disp.flush().unwrap();
    }
}
