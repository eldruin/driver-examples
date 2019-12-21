//! Continuously measure the ultraviolet A and ultraviolet B light sensor data
//! and print it to an SSD1306 OLED display together with the calculated
//! UV index.
//! 
//! Introductory blog post with some pictures here:
//! https://blog.eldruin.com/veml6075-uva-uvb-uv-index-light-sensor-driver-in-rust/
//!
//! This example is runs on the STM32F3 Discovery board using I2C1.
//!
//! ```
//! F3   <-> VEML6075 <-> Display
//! GND  <-> GND      <-> GND
//! 3.3V <-> VCC      <-> VDD
//! PB7  <-> SDA      <-> SDA
//! PB6  <-> SCL      <-> SCL
//! ```
//!
//! Beware that the VEML6075 runs on 3.3V but PB6 and PB7 run on 5V level
//! so make sure to put a logic level shifter in between.
//!
//! Run with:
//! `cargo run --example veml6075-uv-display-f3 --target thumbv7em-none-eabihf`,

#![deny(unsafe_code)]
#![no_std]
#![no_main]

// panic handler
extern crate embedded_graphics;
extern crate panic_semihosting;

use cortex_m_rt::entry;
use embedded_graphics::fonts::Font6x8;
use embedded_graphics::prelude::*;
use f3::{
    hal::{delay::Delay, i2c::I2c, prelude::*, stm32f30x},
    led::Led,
};
use ssd1306::prelude::*;
use ssd1306::Builder;
use veml6075::{Calibration, Measurement, Veml6075};

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

    let mut sensor = Veml6075::new(manager.acquire(), Calibration::default());

    let mut lines: [heapless::String<heapless::consts::U32>; 3] = [
        heapless::String::new(),
        heapless::String::new(),
        heapless::String::new(),
    ];
    sensor.enable().unwrap();
    loop {
        // Blink LED 0 to check that everything is actually running.
        // If the LED 0 is off, something went wrong.
        led.on();
        delay.delay_ms(50_u16);
        led.off();
        delay.delay_ms(50_u16);

        // If there was an error, it will print 0.00, 0.00, 0.00.
        let Measurement { uva, uvb, uv_index } = sensor.read().unwrap_or(Measurement {
            uva: 0.0,
            uvb: 0.0,
            uv_index: 0.0,
        });

        lines[0].clear();
        lines[1].clear();
        lines[2].clear();

        write!(lines[0], "UVA: {}     ", uva).unwrap();
        write!(lines[1], "UVB: {}     ", uvb).unwrap();
        write!(lines[2], "UV index: {}     ", uv_index).unwrap();
        disp.draw(
            Font6x8::render_str(&lines[0])
                .with_stroke(Some(1u8.into()))
                .into_iter(),
        );
        disp.draw(
            Font6x8::render_str(&lines[1])
                .with_stroke(Some(1u8.into()))
                .translate(Coord::new(0, 12))
                .into_iter(),
        );
        disp.draw(
            Font6x8::render_str(&lines[2])
                .with_stroke(Some(1u8.into()))
                .translate(Coord::new(0, 24))
                .into_iter(),
        );
        disp.flush().unwrap();
    }
}
