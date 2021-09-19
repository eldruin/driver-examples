//! Loop setting a position from 0 to 255 to the channel 0 of a MCP42010
//! digital potentiometer and its inverse to channel 1.
//! The MCP42010 device channels are configured as voltage dividers.
//! These voltages will then be measured by the ADS1115 analog/digital
//! converter and will be printed to the SSD1306 OLED display.
//!
//! This example is runs on the STM32F3 Discovery board using SPI1 and I2C1.
//!
//! ```
//! F3   <-> MCP42x <-> ADS1115 <-> Display
//! GND  <-> VSS    <-> GND     <-> GND
//! GND  <-> PA0
//! GND  <-> PA1
//! +5V  <-> VDD    <-> +5V     <-> +5V
//! +5V  <-> PB0
//! +5V  <-> PB1
//! PA5  <-> CLK
//! PA7  <-> SI
//! PB5  <-> CS
//! PB7             <-> SDA     <-> SDA
//! PB6             <-> SCL     <-> SCL
//!          PW0    <-> A0
//!          PW1    <-> A1
//! ```
//!
//! Run with:
//! `cargo run --example mcp42x-ads1115-display-f3 --target thumbv7em-none-eabihf`,

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
use panic_rtt_target as _;
use rtt_target::{rprintln, rtt_init_print};
use ssd1306::{prelude::*, I2CDisplayInterface, Ssd1306};
use stm32f3xx_hal::{
    self as hal,
    delay::Delay,
    pac,
    prelude::*,
    spi::{config::Config, Spi},
};

use ads1x1x::{channel as AdcChannel, Ads1x1x, FullScaleRange, SlaveAddr};
use core::fmt::Write;
use embedded_hal::adc::OneShot;
use embedded_hal::blocking::delay::DelayMs;
use embedded_hal::digital::v2::OutputPin;
use nb::block;

use mcp4x::{Channel as DigiPotChannel, Mcp4x, MODE};

#[entry]
fn main() -> ! {
    rtt_init_print!();
    rprintln!("MCP42010 example");

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
    let mut gpioa = dp.GPIOA.split(&mut rcc.ahb);

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

    let mut adc = Ads1x1x::new_ads1115(manager.acquire(), SlaveAddr::default());
    // need to be able to measure [0-5V]
    adc.set_full_scale_range(FullScaleRange::Within6_144V)
        .unwrap();

    // SPI configuration
    let sck = gpioa
        .pa5
        .into_af5_push_pull(&mut gpioa.moder, &mut gpioa.otyper, &mut gpioa.afrl);
    let miso = gpioa
        .pa6
        .into_af5_push_pull(&mut gpioa.moder, &mut gpioa.otyper, &mut gpioa.afrl);
    let mosi = gpioa
        .pa7
        .into_af5_push_pull(&mut gpioa.moder, &mut gpioa.otyper, &mut gpioa.afrl);

    let spi_config = Config::default().frequency(1.MHz()).mode(MODE);
    let spi = Spi::new(
        dp.SPI1,
        (sck, miso, mosi),
        spi_config,
        clocks,
        &mut rcc.apb2,
    );

    let mut chip_select = gpiob
        .pb5
        .into_push_pull_output(&mut gpiob.moder, &mut gpiob.otyper);

    chip_select.set_high().unwrap();

    let mut digipot = Mcp4x::new_mcp42x(spi, chip_select);

    let mut position = 0;
    loop {
        // Blink LED 0 to check that everything is actually running.
        // If the LED 0 does not blink, something went wrong.
        led.set_high().unwrap();
        delay.delay_ms(50_u16);
        led.set_low().unwrap();

        // set positions to the digital potentiometer channels
        digipot.set_position(DigiPotChannel::Ch0, position).unwrap();
        digipot
            .set_position(DigiPotChannel::Ch1, 255 - position)
            .unwrap();

        // Read voltage in channel 0 and 1
        let value_ch0 = block!(adc.read(&mut AdcChannel::SingleA0)).unwrap();
        let value_ch1 = block!(adc.read(&mut AdcChannel::SingleA1)).unwrap();

        // make the numbers smaller for reading ease
        let value_ch0 = value_ch0 >> 5;
        let value_ch1 = value_ch1 >> 5;

        let mut lines: [heapless::String<32>; 2] =
            [heapless::String::new(), heapless::String::new()];

        // write some extra spaces after the number to clear up when the numbers get smaller
        write!(lines[0], "Channel 0: {}", value_ch0).unwrap();
        write!(lines[1], "Channel 1: {}", value_ch1).unwrap();

        // print
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

        if position >= 248 {
            position = 0
        } else {
            position += 8;
        }
    }
}
