//! Continuously read the temperature with the LM75 and display it in
//! an SSD1306 OLED display.
//!
//! This example is runs on the STM32F3 Discovery board using I2C1.
//!
//! ```
//! F3  <-> LM75 <-> Display
//! GND <-> GND  <-> GND
//! VCC <-> +5V  <-> +5V
//! PB7 <-> SDA  <-> SDA
//! PB6 <-> SCL  <-> SCL
//! ```
//! Run with:
//! `cargo run --example lm75-display-f3 --target thumbv7em-none-eabihf`,
//! currently only works on nightly.

#![deny(unsafe_code)]
#![no_std]
#![no_main]

// panic handler
extern crate embedded_graphics;
extern crate panic_semihosting;

use cortex_m_rt::entry;
use embedded_graphics::{
    fonts::{Font6x8, Text},
    pixelcolor::BinaryColor,
    prelude::*,
    style::TextStyleBuilder,
};
use f3::{
    hal::{delay::Delay, i2c::I2c, prelude::*, stm32f30x},
    led::Led,
};
use lm75::{Lm75, SlaveAddr};
use ssd1306::prelude::*;
use ssd1306::Builder;

use core::fmt::Write;
#[entry]
fn main() -> ! {
    let cp = cortex_m::Peripherals::take().unwrap();
    let dp = stm32f30x::Peripherals::take().unwrap();

    let mut flash = dp.FLASH.constrain();
    let mut rcc = dp.RCC.constrain();
    let mut gpioe = dp.GPIOE.split(&mut rcc.ahb);
    let clocks = rcc.cfgr.freeze(&mut flash.acr);

    let mut led: Led = gpioe
        .pe9
        .into_push_pull_output(&mut gpioe.moder, &mut gpioe.otyper)
        .into();
    let mut delay = Delay::new(cp.SYST, clocks);

    let mut gpiob = dp.GPIOB.split(&mut rcc.ahb);
    let scl = gpiob.pb6.into_af4(&mut gpiob.moder, &mut gpiob.afrl);
    let sda = gpiob.pb7.into_af4(&mut gpiob.moder, &mut gpiob.afrl);

    let i2c = I2c::i2c1(dp.I2C1, (scl, sda), 100.khz(), clocks, &mut rcc.apb1);

    let manager = shared_bus::BusManager::<cortex_m::interrupt::Mutex<_>, _>::new(i2c);
    let mut disp: GraphicsMode<_> = Builder::new().connect_i2c(manager.acquire()).into();
    disp.init().unwrap();
    disp.flush().unwrap();

    let text_style = TextStyleBuilder::new(Font6x8)
        .text_color(BinaryColor::On)
        .build();

    let mut lm75 = Lm75::new(manager.acquire(), SlaveAddr::default());

    loop {
        // Blink LED 0 to check that everything is actually running.
        // If the LED 0 is off, something went wrong.
        led.on();
        delay.delay_ms(50_u16);
        led.off();
        delay.delay_ms(50_u16);

        let mut buffer: heapless::String<heapless::consts::U32> = heapless::String::new();
        let temp = lm75.read_temperature().unwrap();

        write!(buffer, "Temperature {}ºC", temp).unwrap();
        disp.clear();
        Text::new(&buffer, Point::zero())
            .into_styled(text_style)
            .draw(&mut disp)
            .unwrap();
        disp.flush().unwrap();
    }
}
