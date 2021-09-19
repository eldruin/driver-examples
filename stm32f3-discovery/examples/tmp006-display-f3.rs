//! Continuously read the object temperature with the TMP006 and display it in
//! an SSD1306 OLED display.
//!
//! Introductory blog post with some pictures here:
//! https://blog.eldruin.com/tmp006-contact-less-infrared-ir-thermopile-driver-in-rust/
//!
//! This example is runs on the STM32F3 Discovery board using I2C1.
//!
//! ```
//! F3  <-> TMP006 <-> Display
//! GND <-> GND    <-> GND
//! VCC <-> +5V    <-> +5V
//! PB7 <-> SDA    <-> SDA
//! PB6 <-> SCL    <-> SCL
//! ```
//! Run with:
//! `cargo run --example tmp006-display-f3 --target thumbv7em-none-eabihf`

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
use nb::block;
use panic_rtt_target as _;
use rtt_target::{rprintln, rtt_init_print};
use ssd1306::{prelude::*, I2CDisplayInterface, Ssd1306};
use stm32f3xx_hal::{self as hal, delay::Delay, pac, prelude::*};

use tmp006::{SlaveAddr, Tmp006};

use core::fmt::Write;
#[entry]
fn main() -> ! {
    rtt_init_print!();
    rprintln!("TMP006 example");

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

    let manager = shared_bus::BusManager::<cortex_m::interrupt::Mutex<_>, _>::new(i2c);
    let interface = I2CDisplayInterface::new(manager.acquire());
    let mut disp = Ssd1306::new(interface, DisplaySize128x64, DisplayRotation::Rotate0)
        .into_buffered_graphics_mode();
    disp.init().unwrap();
    disp.flush().unwrap();

    let text_style = MonoTextStyleBuilder::new()
        .font(&FONT_6X10)
        .text_color(BinaryColor::On)
        .build();

    let mut tmp006 = Tmp006::new(manager.acquire(), SlaveAddr::default());

    let mut lines: [heapless::String<32>; 2] = [heapless::String::new(), heapless::String::new()];
    loop {
        // Blink LED 0 to check that everything is actually running.
        // If the LED 0 is off, something went wrong.
        led.set_high().unwrap();
        delay.delay_ms(50_u16);
        led.set_low().unwrap();
        delay.delay_ms(50_u16);
        lines[0].clear();
        lines[1].clear();

        let calibration_factor = 6e-14;
        let temp_k = block!(tmp006.read_object_temperature(calibration_factor)).unwrap();
        let temp_c = temp_k - 273.15;
        write!(lines[0], "Temperature: {:.2}ºC", temp_c).unwrap();

        // Read data in raw format
        let raw_data = block!(tmp006.read_sensor_data()).unwrap();
        write!(
            lines[1],
            "OV: {}, AT: {}",
            raw_data.object_voltage, raw_data.ambient_temperature
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
