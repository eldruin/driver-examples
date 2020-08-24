//! Continuously measure the CO2 and TVOC equivalents in the air with an
//! iAQ-Core-C module and print the values to an SSD1306 OLED display.
//!
//! This example is runs on the STM32F103 "Bluepill" board using I2C1.
//!
//! ```
//! BP   <-> iAQ-Core-C <-> Display
//! GND  <-> GND        <-> GND
//! 3.3V <-> VCC        <-> VDD
//! PB8  <-> SCL        <-> SCL
//! PB9  <-> SDA        <-> SDA
//! ```
//!
//! Run with:
//! `cargo run --example iaq-core-c-gas-voc-display-bp`,

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
use panic_semihosting as _;
use ssd1306::{prelude::*, Builder, I2CDIBuilder};
use stm32f1xx_hal::{
    delay::Delay,
    i2c::{BlockingI2c, DutyCycle, Mode},
    pac,
    prelude::*,
};

use iaq_core::{IaqCore, Measurement};

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
        Mode::Standard {
            frequency: 100_000.hz(),
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
    let interface = I2CDIBuilder::new().init(manager.acquire());
    let mut disp: GraphicsMode<_> = Builder::new().connect(interface).into();
    disp.init().unwrap();
    disp.flush().unwrap();

    let text_style = TextStyleBuilder::new(Font6x8)
        .text_color(BinaryColor::On)
        .build();

    let mut sensor = IaqCore::new(manager.acquire());
    let mut lines: [heapless::String<heapless::consts::U32>; 3] = [
        heapless::String::new(),
        heapless::String::new(),
        heapless::String::new(),
    ];
    loop {
        // Blink LED 0 to check that everything is actually running.
        // If the LED 0 is off, something went wrong.
        led.set_high().unwrap();
        delay.delay_ms(50_u16);
        led.set_low().unwrap();
        delay.delay_ms(50_u16);

        let data = block!(sensor.data()).unwrap_or(Measurement::default());

        lines[0].clear();
        lines[1].clear();
        lines[2].clear();
        write!(lines[0], "CO2: {} ppm  ", data.co2).unwrap();
        write!(lines[1], "TVOC: {} ppb  ", data.tvoc).unwrap();
        write!(lines[2], "Resistance: {} Ohm  ", data.resistance).unwrap();
        disp.clear();
        for (i, line) in lines.iter().enumerate() {
            Text::new(line, Point::new(0, i as i32 * 16))
                .into_styled(text_style)
                .draw(&mut disp)
                .unwrap();
        }
        disp.flush().unwrap();
    }
}
