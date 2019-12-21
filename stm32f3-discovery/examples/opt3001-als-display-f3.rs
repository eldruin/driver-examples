//! Continuously measure the ambient light sensor data with an OPT3001
//! and print it to an SSD1306 OLED display in lux.
//! 
//! Introductory blog post with some pictures here:
//! https://blog.eldruin.com/opt300x-ambient-light-sensor-driver-in-rust/
//!
//! This example is runs on the STM32F3 Discovery board using I2C1.
//!
//! ```
//! F3   <-> OPT3001 <-> Display
//! GND  <-> GND     <-> GND
//! 3.3V <-> VCC     <-> VDD
//! PB7  <-> SDA     <-> SDA
//! PB6  <-> SCL     <-> SCL
//! ```
//!
//! Beware that the OPT3001 runs on 3.3V but PB6 and PB7 run on 5V level
//! so make sure to put a logic level shifter in between.
//!
//! Run with:
//! `cargo run --example opt3001-als-display-f3 --target thumbv7em-none-eabihf`,

#![deny(unsafe_code)]
#![no_std]
#![no_main]

// panic handler
use nb::block;
use panic_semihosting as _;

use cortex_m_rt::entry;
use embedded_graphics::fonts::Font6x8;
use embedded_graphics::prelude::*;
use f3::{
    hal::{delay::Delay, i2c::I2c, prelude::*, stm32f30x},
    led::Led,
};
use opt300x::{Opt300x, SlaveAddr};
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

    let mut sensor = Opt300x::new_opt3001(manager.acquire(), SlaveAddr::default());
    let mut buffer: heapless::String<heapless::consts::U64> = heapless::String::new();
    loop {
        // Blink LED 0 to check that everything is actually running.
        // If the LED 0 is off, something went wrong.
        led.on();
        delay.delay_ms(50_u16);
        led.off();
        delay.delay_ms(50_u16);

        // If there is an error, it will print -1.0
        let m = block!(sensor.read_lux()).unwrap();

        buffer.clear();
        write!(buffer, "lux {:2}     ", m.result).unwrap();

        disp.draw(
            Font6x8::render_str(&buffer)
                .with_stroke(Some(1u8.into()))
                .into_iter(),
        );
        disp.flush().unwrap();
    }
}
