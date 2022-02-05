//! Continuously measure the ambient light sensor data with an OPT3001
//! and print it to an SSD1306 OLED display in lux.
//!
//! Introductory blog post with some pictures here:
//! https://blog.eldruin.com/opt300x-ambient-light-sensor-driver-in-rust/
//!
//! This example is runs on the STM32F103 "Bluepill" board using I2C1.
//!
//! ```
//! BP   <-> OPT3001 <-> Display
//! GND  <-> GND     <-> GND
//! 3.3V <-> VCC     <-> VDD
//! PB8  <-> SCL     <-> SCL
//! PB9  <-> SDA     <-> SDA
//! ```
//!
//! Run with:
//! `cargo embed --example opt3001-als-display-bp`,

#![deny(unsafe_code)]
#![no_std]
#![no_main]

use core::fmt::Write;
use cortex_m_rt::entry;
use embedded_graphics::{
    fonts::{Font6x8, Text},
    pixelcolor::BinaryColor,
    prelude::*,
    style::TextStyleBuilder,
};
use embedded_hal::digital::v2::OutputPin;
use nb::block;
use opt300x::{Measurement, Opt300x, SlaveAddr, Status};
use panic_rtt_target as _;
use rtt_target::{rprintln, rtt_init_print};
use ssd1306::{prelude::*, Builder, I2CDIBuilder};
use stm32f1xx_hal::{
    delay::Delay,
    i2c::{BlockingI2c, DutyCycle, Mode},
    pac,
    prelude::*,
};

#[entry]
fn main() -> ! {
    rtt_init_print!();
    rprintln!("OPT3001 example");
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
            frequency: 100_000.hz(),
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

    let manager = shared_bus::BusManagerSimple::new(i2c);
    let interface = I2CDIBuilder::new().init(manager.acquire_i2c());
    let mut disp: GraphicsMode<_> = Builder::new().connect(interface).into();
    disp.init().unwrap();
    disp.flush().unwrap();

    let text_style = TextStyleBuilder::new(Font6x8)
        .text_color(BinaryColor::On)
        .build();

    let mut sensor = Opt300x::new_opt3001(manager.acquire_i2c(), SlaveAddr::Alternative(false, false));

    let mut buffer: heapless::String<64> = heapless::String::new();
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
        write!(buffer, "lux: {:.2}", m.result).unwrap();
        disp.clear();
        Text::new(&buffer, Point::zero())
            .into_styled(text_style)
            .draw(&mut disp)
            .unwrap();

        disp.flush().unwrap();
    }
}
