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
//! `cargo run --example f3-mcp42x-ads1115-display --target thumbv7em-none-eabihf`,

#![deny(unsafe_code)]
#![no_std]
#![no_main]

// panic handler
extern crate panic_semihosting;
extern crate embedded_graphics;

use cortex_m_rt::entry;
use f3::{
    hal::{delay::Delay, spi::Spi, i2c::I2c, prelude::*, stm32f30x},
    led::Led,
};
use embedded_hal::adc::OneShot;
use ads1x1x::{channel as AdcChannel, Ads1x1x, FullScaleRange, SlaveAddr};
use embedded_graphics::fonts::Font6x8;
use embedded_graphics::prelude::*;
use ssd1306::prelude::*;
use ssd1306::Builder;
use nb::block;

use core::fmt::Write;

use mcp4x::{Channel as DigiPotChannel, Mcp4x, MODE};

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
    let mut gpioa = dp.GPIOA.split(&mut rcc.ahb);

    let scl = gpiob.pb6.into_af4(&mut gpiob.moder, &mut gpiob.afrl);
    let sda = gpiob.pb7.into_af4(&mut gpiob.moder, &mut gpiob.afrl);

    let i2c = I2c::i2c1(dp.I2C1, (scl, sda), 100.khz(), clocks, &mut rcc.apb1);

    let manager = shared_bus::BusManager::<cortex_m::interrupt::Mutex<_>, _>::new(i2c);
    let mut disp: GraphicsMode<_> = Builder::new().connect_i2c(manager.acquire()).into();

    disp.init().unwrap();
    disp.flush().unwrap();

    let mut adc = Ads1x1x::new_ads1115(manager.acquire(), SlaveAddr::default());
    // need to be able to measure [0-5V]
    adc.set_full_scale_range(FullScaleRange::Within6_144V).unwrap();

    // SPI configuration
    let sck = gpioa.pa5.into_af5(&mut gpioa.moder, &mut gpioa.afrl);
    let miso = gpioa.pa6.into_af5(&mut gpioa.moder, &mut gpioa.afrl);
    let mosi = gpioa.pa7.into_af5(&mut gpioa.moder, &mut gpioa.afrl);

    let spi = Spi::spi1(
        dp.SPI1,
        (sck, miso, mosi),
        MODE,
        1.mhz(),
        clocks,
        &mut rcc.apb2,
    );

    let mut chip_select = gpiob
        .pb5
        .into_push_pull_output(&mut gpiob.moder, &mut gpiob.otyper);

    chip_select.set_high();

    let mut digipot = Mcp4x::new_mcp42x(spi, chip_select);


    let mut position = 0;
    loop {
        // Blink LED 0 to check that everything is actually running.
        // If the LED 0 does not blink, something went wrong.
        led.on();
        delay.delay_ms(50_u16);
        led.off();

        // set positions to the digital potentiometer channels
        digipot.set_position(DigiPotChannel::Ch0, position).unwrap();
        digipot.set_position(DigiPotChannel::Ch1, 255-position).unwrap();

        // Read voltage in channel 0 and 1
        let value_ch0 = block!(adc.read(&mut AdcChannel::SingleA0)).unwrap();
        let value_ch1 = block!(adc.read(&mut AdcChannel::SingleA1)).unwrap();

        // make the numbers smaller for reading ease
        let value_ch0 = value_ch0 >> 5;
        let value_ch1 = value_ch1 >> 5;

        let mut line0: heapless::String<heapless::consts::U32> = heapless::String::new();
        let mut line1: heapless::String<heapless::consts::U32> = heapless::String::new();

        // write some extra spaces after the number to clear up when the numbers get smaller
        write!(line0, "Channel 0: {}    ", value_ch0).unwrap();
        write!(line1, "Channel 1: {}    ", value_ch1).unwrap();

        // print first line
        disp.draw(
        Font6x8::render_str(&line0)
            .with_stroke(Some(1u8.into()))
            .into_iter(),
        );
        // print second line
        disp.draw(
        Font6x8::render_str(&line1)
            .with_stroke(Some(1u8.into()))
            .translate(Coord::new(0, 16))
            .into_iter(),
        );
        disp.flush().unwrap();

        if position >= 248 {
            position = 0
        }
        else {
            position += 8;
        }
    }
}
