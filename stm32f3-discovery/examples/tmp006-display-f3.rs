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

use cortex_m_rt::entry;
use embedded_graphics::{fonts::Font6x8, prelude::*};
use f3::{
    hal::{delay::Delay, i2c::I2c, prelude::*, stm32f30x},
    led::Led,
};
use nb::block;
use panic_semihosting as _;
use ssd1306::{prelude::*, Builder};
use tmp006::{SlaveAddr, Tmp006};

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

    let mut tmp006 = Tmp006::new(manager.acquire(), SlaveAddr::default());

    let mut lines: [heapless::String<heapless::consts::U32>; 2] =
        [heapless::String::new(), heapless::String::new()];
    loop {
        // Blink LED 0 to check that everything is actually running.
        // If the LED 0 is off, something went wrong.
        led.on();
        delay.delay_ms(50_u16);
        led.off();
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
        disp.flush().unwrap();
    }
}
