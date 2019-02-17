//! Loop setting a position from 0 to 4095 to the channel 0 of a MCP4921
//! digital-to-analog converter.
//! The voltage output of the MCP4921 device will then be measured by the
//! ADS1115 analog-to-digital converter and will be printed to the
//! SSD1306 OLED display.
//!
//! This example is runs on the STM32F3 Discovery board using SPI1 and I2C1.
//!
//! ```
//! F3   <-> MCP4921 <-> ADS1115 <-> Display
//! GND  <-> VSS     <-> GND     <-> GND
//! GND  <-> LDAC
//! +5V  <-> VDD     <-> +5V     <-> +5V
//! +5V  <-> VREFA
//! PA5  <-> CLK
//! PA7  <-> SI
//! PB5  <-> CS
//! PB7              <-> SDA     <-> SDA
//! PB6              <-> SCL     <-> SCL
//!          VOUTA   <-> A0
//! ```
//!
//! Run with:
//! `cargo run --example mcp4921-ads1115-display-f3 --target thumbv7em-none-eabihf`,

#![deny(unsafe_code)]
#![no_std]
#![no_main]

// panic handler
extern crate embedded_graphics;
extern crate panic_semihosting;

use ads1x1x::{channel as AdcChannel, Ads1x1x, FullScaleRange, SlaveAddr};
use cortex_m_rt::entry;
use embedded_graphics::fonts::Font6x8;
use embedded_graphics::prelude::*;
use embedded_hal::adc::OneShot;
use f3::{
    hal::{delay::Delay, i2c::I2c, prelude::*, spi::Spi, stm32f30x},
    led::Led,
};
use nb::block;
use ssd1306::prelude::*;
use ssd1306::Builder;

use core::fmt::Write;

use mcp49xx::{Command as DacCommand, Mcp49xx, MODE0};

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
    // need to be able to measure [0-5V] since that is the reference voltage of the DAC (VREFA)
    adc.set_full_scale_range(FullScaleRange::Within6_144V)
        .unwrap();

    // SPI configuration
    let sck = gpioa.pa5.into_af5(&mut gpioa.moder, &mut gpioa.afrl);
    let miso = gpioa.pa6.into_af5(&mut gpioa.moder, &mut gpioa.afrl);
    let mosi = gpioa.pa7.into_af5(&mut gpioa.moder, &mut gpioa.afrl);

    let spi = Spi::spi1(
        dp.SPI1,
        (sck, miso, mosi),
        MODE0,
        1.mhz(),
        clocks,
        &mut rcc.apb2,
    );

    let mut chip_select = gpiob
        .pb5
        .into_push_pull_output(&mut gpiob.moder, &mut gpiob.otyper);

    chip_select.set_high();

    let mut dac = Mcp49xx::new_mcp4921(spi, chip_select);
    let dac_cmd = DacCommand::default();
    let mut position = 0;
    loop {
        // Blink LED 0 to check that everything is actually running.
        // If the LED 0 does not blink, something went wrong.
        led.on();
        delay.delay_ms(50_u16);
        led.off();

        dac.send(dac_cmd.value(position)).unwrap();

        // Read voltage in channel 0
        let value_ch0 = block!(adc.read(&mut AdcChannel::SingleA0)).unwrap();

        // make the number smaller for reading ease
        let value_ch0 = value_ch0 >> 5;

        let mut msg: heapless::String<heapless::consts::U64> = heapless::String::new();

        // write some extra spaces after the number to clear up when the number get smaller
        write!(msg, "Channel 0: {}   ", value_ch0).unwrap();

        // print
        disp.draw(
            Font6x8::render_str(&msg)
                .with_stroke(Some(1u8.into()))
                .into_iter(),
        );
        disp.flush().unwrap();

        // Actually this gets only until 4080.
        // Then it would be too big so we set it to 0.
        position += 255;
        if position >= 1 << 12 {
            position = 0
        }
    }
}
