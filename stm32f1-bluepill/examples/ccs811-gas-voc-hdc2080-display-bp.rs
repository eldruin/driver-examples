//! Continuously measure the eCO2 and eTVOC in the air and print it to an
//! SSD1306 OLED display.
//! In order to compensate for the ambient temperature and humidity, an HDC2080
//! sensor is used.
//!
//! This example is runs on the STM32F103 "Bluepill" board using I2C1.
//!
//! ```
//! BP   <-> CCS811 <-> HDC2080 <-> Display
//! GND  <-> GND    <-> GND     <-> GND
//! 3.3V <-> VCC    <-> VCC     <-> VDD
//! PB8  <-> SCL    <-> SCL     <-> SCL
//! PB9  <-> SDA    <-> SDA     <-> SDA
//! GND  <-> nWAKE
//! 3.3V <-> RST
//! ```
//!
//! Run with:
//! `cargo run --example ccs811-gas-voc-display-bp`,

#![deny(unsafe_code)]
#![no_std]
#![no_main]

use core::fmt::Write;
use cortex_m_rt::entry;
use embedded_ccs811::{
    prelude::*, AlgorithmResult, Ccs811Awake, MeasurementMode, SlaveAddr as Ccs811SlaveAddr,
};
use embedded_graphics::{
    fonts::{Font6x8, Text},
    pixelcolor::BinaryColor,
    prelude::*,
    style::TextStyleBuilder,
};
use embedded_hal::digital::v2::OutputPin;
use hdc20xx::{Hdc20xx, SlaveAddr as Hdc20xxSlaveAddr};
use heapless::String;
use nb::block;
use panic_semihosting as _;
use ssd1306::{prelude::*, Builder, I2CDIBuilder};
use stm32f1xx_hal::{
    delay::Delay,
    i2c::{BlockingI2c, DutyCycle, Mode},
    pac,
    prelude::*,
};

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
    let mut nwake = gpiob.pb7.into_push_pull_output(&mut gpiob.crl);
    nwake.set_high().unwrap();

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

    let manager = shared_bus::BusManager::<cortex_m::interrupt::Mutex<_>, _>::new(i2c);
    let interface = I2CDIBuilder::new().init(manager.acquire());
    let mut disp: GraphicsMode<_> = Builder::new().connect(interface).into();
    disp.init().unwrap();
    disp.flush().unwrap();

    let text_style = TextStyleBuilder::new(Font6x8)
        .text_color(BinaryColor::On)
        .build();

    let mut hdc2080 = Hdc20xx::new(manager.acquire(), Hdc20xxSlaveAddr::default());
    let mut ccs811 = Ccs811Awake::new(manager.acquire(), Ccs811SlaveAddr::default());
    ccs811.software_reset().unwrap();
    delay.delay_ms(10_u16);
    let mut lines: [String<heapless::consts::U32>; 4] =
        [String::new(), String::new(), String::new(), String::new()];

    let mut ccs811 = ccs811.start_application().ok().unwrap();
    let mut env = block!(hdc2080.read()).unwrap();
    ccs811
        .set_environment(env.temperature, env.humidity.unwrap_or(0.0))
        .unwrap();
    ccs811.set_mode(MeasurementMode::ConstantPower1s).unwrap();

    let default = AlgorithmResult {
        eco2: 9999,
        etvoc: 9999,
        raw_current: 255,
        raw_voltage: 9999,
    };

    let mut counter = 0;
    loop {
        // Blink LED 0 to check that everything is actually running.
        // If the LED 0 is off, something went wrong.
        led.set_high().unwrap();
        delay.delay_ms(500_u16);
        led.set_low().unwrap();
        delay.delay_ms(500_u16);

        let data = block!(ccs811.data()).unwrap_or(default);

        counter += 1;
        if counter > 10 {
            counter = 0;

            env = block!(hdc2080.read()).unwrap();
            ccs811
                .set_environment(env.temperature, env.humidity.unwrap_or(0.0))
                .unwrap();
        }

        for i in 0..4 {
            lines[i].clear();
        }
        write!(lines[0], "eCO2: {}", data.eco2).unwrap();
        write!(lines[1], "eTVOC: {}", data.etvoc).unwrap();
        write!(lines[2], "Temp: {:.2}ºC", env.temperature).unwrap();
        write!(lines[3], "Humidity: {:.2}%", env.humidity.unwrap_or(0.0)).unwrap();
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
