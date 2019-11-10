//! Continuously measure the ambient light sensor data with an OPT3001
//! and print it to an SSD1306 OLED display in lux.
//!
//! This example is runs on the STM32F103 "Bluepill" board using I2C1.
//!
//! ```
//! F3   <-> OPT3001 <-> Display
//! GND  <-> GND     <-> GND
//! 3.3V <-> VCC     <-> VDD
//! PB8  <-> SCL     <-> SCL
//! PB9  <-> SDA     <-> SDA
//! ```
//!
//! Run with:
//! `cargo run --example opt3001-als-display-bp`,

#![deny(unsafe_code)]
#![no_std]
#![no_main]

extern crate embedded_graphics;
use cortex_m_rt::entry;
use embedded_graphics::fonts::Font6x8;
use embedded_graphics::prelude::*;
use embedded_hal::digital::v2::OutputPin;
use nb::block;
use panic_semihosting as _;
use ssd1306::prelude::*;
use ssd1306::Builder;

use opt300x::{Measurement, Opt300x, SlaveAddr, Status};
use stm32f1xx_hal::{
    delay::Delay,
    i2c::{BlockingI2c, DutyCycle, Mode},
    pac,
    prelude::*,
};

use core::fmt::Write;
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
            frequency: 100_000,
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

    let manager = shared_bus::BusManager::<cortex_m::interrupt::Mutex<_>, _>::new(i2c);
    let mut disp: GraphicsMode<_> = Builder::new().connect_i2c(manager.acquire()).into();

    disp.init().unwrap();
    disp.flush().unwrap();

    let mut sensor = Opt300x::new_opt3001(manager.acquire(), SlaveAddr::Alternative(false, false));

    let mut buffer: heapless::String<heapless::consts::U64> = heapless::String::new();
    loop {
        // Blink LED 0 to check that everything is actually running.
        // If the LED 0 is off, something went wrong.
        led.set_high().unwrap();
        delay.delay_ms(50_u16);
        led.set_low().unwrap();
        delay.delay_ms(50_u16);

        let def = Measurement {
            result: 999.9,
            status: Status::default(),
        };
        let m = block!(sensor.read_lux()).unwrap_or(def);

        buffer.clear();
        write!(buffer, "lux: {:2}     ", m.result).unwrap();
        disp.draw(
            Font6x8::render_str(&buffer)
                .with_stroke(Some(1u8.into()))
                .into_iter(),
        );
        disp.flush().unwrap();
    }
}
