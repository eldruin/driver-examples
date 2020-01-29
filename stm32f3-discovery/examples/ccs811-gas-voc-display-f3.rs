//! Continuously measure the eCO2 and eTVOC in the air
//! and print it to an SSD1306 OLED display.
//!
//! This example is runs on the STM32F3 Discovery board using I2C1.
//!
//! ```
//! F3   <-> CCS811 <-> Display
//! GND  <-> GND    <-> GND
//! 3.3V <-> VCC    <-> VDD
//! PB6  <-> SCL    <-> SCL
//! PB7  <-> SDA    <-> SDA
//! GND  <-> nWAKE
//! 3.3V <-> RST
//! ```
//!
//! Run with:
//! `cargo run --example ccs811-gas-voc-display-f3`,

#![deny(unsafe_code)]
#![no_std]
#![no_main]

use core::fmt::Write;
use cortex_m_rt::entry;
use embedded_ccs811::{prelude::*, AlgorithmResult, Ccs811Awake, MeasurementMode, SlaveAddr};
use embedded_graphics::{fonts::Font6x12, prelude::*};
use f3::{
    hal::{delay::Delay, i2c::I2c, prelude::*, stm32f30x},
    led::Led,
};
use nb::block;
use panic_semihosting as _;
use ssd1306::{prelude::*, Builder};

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

    let address = SlaveAddr::default();
    let mut sensor = Ccs811Awake::new(manager.acquire(), address);
    sensor.software_reset().unwrap();
    delay.delay_ms(3_u16);
    let mut sensor = sensor.start_application().ok().unwrap();
    delay.delay_ms(2_u8);
    let temperature_c = 25.0;
    let relative_humidity_perc = 60.0;
    sensor
        .set_environment(temperature_c, relative_humidity_perc)
        .unwrap();
    sensor.set_mode(MeasurementMode::ConstantPower1s).unwrap();

    let default = AlgorithmResult {
        eco2: 9999,
        etvoc: 9999,
        raw_current: 255,
        raw_voltage: 9999,
    };
    let mut lines: [heapless::String<heapless::consts::U32>; 2] =
        [heapless::String::new(), heapless::String::new()];
    loop {
        // Blink LED 0 to check that everything is actually running.
        // If the LED 0 is off, something went wrong.
        led.on();
        delay.delay_ms(50_u16);
        led.off();
        delay.delay_ms(50_u16);

        let data = block!(sensor.data()).unwrap_or(default);

        lines[0].clear();
        lines[1].clear();
        write!(lines[0], "eCO2: {}", data.eco2).unwrap();
        write!(lines[1], "eTVOC: {}", data.etvoc).unwrap();
        disp.draw(
            Font6x12::render_str(&lines[0])
                .with_stroke(Some(1u8.into()))
                .into_iter(),
        );
        disp.draw(
            Font6x12::render_str(&lines[1])
                .with_stroke(Some(1u8.into()))
                .translate(Coord::new(0, 16))
                .into_iter(),
        );
        disp.flush().unwrap();
    }
}
