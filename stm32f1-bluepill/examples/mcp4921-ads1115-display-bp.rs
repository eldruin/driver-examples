//! Loop setting a position from 0 to 4095 to the channel 0 of a MCP4921
//! digital-to-analog converter.
//! The voltage output of the MCP4921 device will then be measured by the
//! ADS1115 analog-to-digital converter and will be printed to the
//! SSD1306 OLED display.
//!
//! This example is runs on the STM32F103 "Bluepill" board using SPI1 and I2C1.
//!
//! ```
//! BP   <-> MCP4921 <-> ADS1115 <-> Display
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
//! `cargo run --example mcp4921-ads1115-display-bp  --release`

#![deny(unsafe_code)]
#![no_std]
#![no_main]

use ads1x1x::{channel as AdcChannel, Ads1x1x, FullScaleRange, SlaveAddr};
use core::fmt::Write;
use cortex_m_rt::entry;
use embedded_graphics::{
    fonts::{Font6x8, Text},
    pixelcolor::BinaryColor,
    prelude::*,
    style::TextStyleBuilder,
};
use embedded_hal::adc::OneShot;
use embedded_hal::blocking::delay::DelayMs;
use mcp49xx::{Command as DacCommand, Mcp49xx, MODE_0};
use nb::block;
use panic_rtt_target as _;
use rtt_target::{rprintln, rtt_init_print};
use ssd1306::{prelude::*, Builder, I2CDIBuilder};
use stm32f1xx_hal::{
    delay::Delay,
    i2c::{BlockingI2c, DutyCycle, Mode},
    pac,
    prelude::*,
    spi::Spi,
};

#[entry]
fn main() -> ! {
    rtt_init_print!();
    rprintln!("MCP4921 example");
    let cp = cortex_m::Peripherals::take().unwrap();
    let dp = pac::Peripherals::take().unwrap();

    let mut flash = dp.FLASH.constrain();
    let rcc = dp.RCC.constrain();

    let clocks = rcc.cfgr.freeze(&mut flash.acr);

    let mut afio = dp.AFIO.constrain();
    let mut gpioa = dp.GPIOA.split();
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

    // SPI1
    let sck = gpioa.pa5.into_alternate_push_pull(&mut gpioa.crl);
    let miso = gpioa.pa6;
    let mosi = gpioa.pa7.into_alternate_push_pull(&mut gpioa.crl);
    let mut cs = gpioa.pa4.into_push_pull_output(&mut gpioa.crl);

    let mut spi = Spi::spi1(
        dp.SPI1,
        (sck, miso, mosi),
        &mut afio.mapr,
        MODE_0,
        1_u32.mhz(),
        clocks,
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

    let mut adc = Ads1x1x::new_ads1115(manager.acquire_i2c(), SlaveAddr::default());
    // need to be able to measure [0-5V] since that is the reference voltage of the DAC (VREFA)
    adc.set_full_scale_range(FullScaleRange::Within6_144V)
        .unwrap();

    cs.set_high();

    let mut dac = Mcp49xx::new_mcp4921(cs);
    let dac_cmd = DacCommand::default();
    let mut position = 0;
    loop {
        // Blink LED 0 to check that everything is actually running.
        // If the LED 0 does not blink, something went wrong.
        led.set_high();
        delay.delay_ms(50_u16);
        led.set_low();

        dac.send(&mut spi, dac_cmd.value(position)).unwrap();

        // Read voltage in channel 0
        let value_ch0 = block!(adc.read(&mut AdcChannel::SingleA0)).unwrap();

        // make the number smaller for reading ease
        let value_ch0 = value_ch0 >> 5;

        let mut buffer: heapless::String<64> = heapless::String::new();

        // write some extra spaces after the number to clear up when the number get smaller
        write!(buffer, "Channel 0: {}   ", value_ch0).unwrap();

        // print
        disp.clear();
        Text::new(&buffer, Point::zero())
            .into_styled(text_style)
            .draw(&mut disp)
            .unwrap();
        disp.flush().unwrap();

        // Actually this gets only until 4080.
        // Then it would be too big so we set it to 0.
        position += 255;
        if position >= 1 << 12 {
            position = 0
        }
    }
}
