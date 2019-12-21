//! Continuously measure the ultraviolet A and ultraviolet B light sensor data
//! and print it to an SSD1306 OLED display.
//!
//! This example is runs on the STM32F103 "Bluepill" board using I2C1.
//!
//! ```
//! BP   <-> VEML6070 <-> Display
//! GND  <-> GND      <-> GND
//! 3.3V <-> VCC      <-> VDD
//! PB8  <-> SCL      <-> SCL
//! PB9  <-> SDA      <-> SDA
//! ```
//!
//! Run with:
//! `cargo run --example veml6070-uv-display-bp`,

#![deny(unsafe_code)]
#![no_std]
#![no_main]

extern crate embedded_graphics;
use cortex_m_rt::entry;
use embedded_graphics::fonts::Font6x8;
use embedded_graphics::prelude::*;
use embedded_hal::digital::v2::OutputPin;
use panic_semihosting as _;
use ssd1306::prelude::*;
use ssd1306::Builder;

use stm32f1xx_hal::{
    delay::Delay,
    i2c::{BlockingI2c, DutyCycle, Mode},
    pac,
    prelude::*,
};
use veml6070::VEML6070;

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
            frequency: 400_000,
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

    let mut sensor = VEML6070::new(manager.acquire());

    let mut buffer: heapless::String<heapless::consts::U64> = heapless::String::new();
    sensor.enable().unwrap();
    loop {
        // Blink LED 0 to check that everything is actually running.
        // If the LED 0 is off, something went wrong.
        led.set_high().unwrap();
        delay.delay_ms(50_u16);
        led.set_low().unwrap();
        delay.delay_ms(50_u16);

        // If there was an error, it will print 65535.
        let uva = sensor.read_uv().unwrap_or(65535);

        buffer.clear();
        write!(buffer, "UVA: {}     ", uva).unwrap();
        disp.draw(
            Font6x8::render_str(&buffer)
                .with_stroke(Some(1u8.into()))
                .into_iter(),
        );
        disp.flush().unwrap();
    }
}
