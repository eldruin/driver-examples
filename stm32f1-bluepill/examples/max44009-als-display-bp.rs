//! Continuously measure the ambient light sensor data with a MAX44009
//! and print it to an SSD1306 OLED display in lux.
//!
//! This example is runs on the STM32F103 "Bluepill" board using I2C1.
//!
//! ```
//! BP   <-> MAX44009 <-> Display
//! GND  <-> GND      <-> GND
//! 3.3V <-> VCC      <-> VDD
//! PB8  <-> SCL      <-> SCL
//! PB9  <-> SDA      <-> SDA
//! ```
//!
//! Run with:
//! `cargo embed --example max44009-als-display-bp --release`,

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
use max44009::{Max44009, SlaveAddr};
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
    rprintln!("MAX44009 example");
    let cp = cortex_m::Peripherals::take().unwrap();
    let dp = pac::Peripherals::take().unwrap();

    let mut flash = dp.FLASH.constrain();
    let rcc = dp.RCC.constrain();

    let clocks = rcc.cfgr.freeze(&mut flash.acr);

    let mut afio = dp.AFIO.constrain();

    let mut gpiob = dp.GPIOB.split();

    let scl = gpiob.pb8.into_alternate_open_drain(&mut gpiob.crh);
    let sda = gpiob.pb9.into_alternate_open_drain(&mut gpiob.crh);

    let i2c = BlockingI2c::i2c1(
        dp.I2C1,
        (scl, sda),
        &mut afio.mapr,
        Mode::Fast {
            frequency: 400_000.hz(),
            duty_cycle: DutyCycle::Ratio2to1,
        },
        clocks,
        1000,
        10,
        1000,
        1000,
    );

    let mut gpioc = dp.GPIOC.split();
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

    let mut sensor = Max44009::new(manager.acquire_i2c(), SlaveAddr::default());

    let mut buffer: heapless::String<64> = heapless::String::new();
    loop {
        // Blink LED 0 to check that everything is actually running.
        // If the LED 0 is off, something went wrong.
        led.set_high();
        delay.delay_ms(50_u16);
        led.set_low();
        delay.delay_ms(50_u16);

        // If there is an error, it will print -1.0
        let lux = sensor.read_lux().unwrap_or(-1.0);

        buffer.clear();
        write!(buffer, "lux {:.2}", lux).unwrap();
        disp.clear();
        Text::new(&buffer, Point::zero())
            .into_styled(text_style)
            .draw(&mut disp)
            .unwrap();
        disp.flush().unwrap();
    }
}
