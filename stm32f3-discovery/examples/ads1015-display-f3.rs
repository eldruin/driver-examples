//! Measure the voltages with an ADS1015 analog/digital
//! converter and print them to an SSD1306 OLED display.
//!
//! You can see further explanations about this device and how this example
//! works here:
//!
//! https://blog.eldruin.com/ads1x1x-analog-to-digital-converter-driver-in-rust/
//!
//! This example is runs on the STM32F3 Discovery board using I2C1.
//!
//! ```
//! F3  <-> ADS1015 <-> Display
//! GND <-> GND     <-> GND
//! +5V <-> +5V     <-> +5V
//! PB7 <-> SDA     <-> SDA
//! PB6 <-> SCL     <-> SCL
//! ```
//!
//! For example you can create a simple voltage divider with resistors.
//! The values do not matter much but it is nicer to understand if they are
//! all the same as the voltage will be divided equally.
//! I used 3 resistors of 20KOhm and the inputs of the ADC were connected
//! as follows:
//!
//! ```
//!       ADS1015
//! +5V <-> A0
//!  |
//!  R3
//!  |  <-> A1
//!  R2
//!  |  <-> A2
//!  R1
//!  |
//! GND <-> A3
//! ```
//!
//! You can see an image of this [here](https://github.com/eldruin/driver-examples/blob/master/media/ads1015-voltage-divider.jpg).
//!
//! With this setup we should get the reading for +5V on channel A0,
//! the reading for GND on channel A3 and A1 and A2 equally spaced in between
//! (within resistence tolerances).
//!
//! I get these values:
//! Channel 0: 1575
//! Channel 1: 1051
//! Channel 2: 524
//! Channel 3: 0
//!
//! We can calculate the relations and voltage that correspond to each channel if we
//! assume that 1575 corresponds to 5V.
//!                         Factor        Voltage
//! Channel 0: 1575 / 1575 = 1     * 5V =   5V
//! Channel 1: 1051 / 1575 = 0.667 * 5V =  3.34V
//! Channel 2: 524  / 1575 = 0.333 * 5V =  1.66V
//! Channel 3: 0    / 1575 = 0     * 5V =   0V
//!
//! As you can see, the voltage was divided equally by all resistors.
//!
//! Run with:
//! `cargo run --example ads1015-display-f3 --target thumbv7em-none-eabihf`,

#![deny(unsafe_code)]
#![no_std]
#![no_main]

use core::fmt::Write;
use cortex_m_rt::entry;
use embedded_graphics::{
    mono_font::{ascii::FONT_6X10, MonoTextStyleBuilder},
    pixelcolor::BinaryColor,
    prelude::*,
    text::{Baseline, Text},
};
use embedded_hal::adc::OneShot;
use nb::block;
use panic_rtt_target as _;
use rtt_target::{rprintln, rtt_init_print};
use ssd1306::{prelude::*, I2CDisplayInterface, Ssd1306};
use stm32f3xx_hal::{self as hal, delay::Delay, pac, prelude::*};

use ads1x1x::{channel as AdcChannel, Ads1x1x, FullScaleRange, SlaveAddr};

#[entry]
fn main() -> ! {
    rtt_init_print!();
    rprintln!("ADS1015 example");

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

    let manager = shared_bus::BusManagerSimple::new(i2c);
    let interface = I2CDisplayInterface::new(manager.acquire_i2c());
    let mut disp = Ssd1306::new(interface, DisplaySize128x64, DisplayRotation::Rotate0)
        .into_buffered_graphics_mode();
    disp.init().unwrap();
    disp.flush().unwrap();

    let text_style = MonoTextStyleBuilder::new()
        .font(&FONT_6X10)
        .text_color(BinaryColor::On)
        .build();

    led.set_high().unwrap();
    delay.delay_ms(50_u16);
    led.set_low().unwrap();
    let mut adc = Ads1x1x::new_ads1015(manager.acquire_i2c(), SlaveAddr::default());
    // need to be able to measure [0-5V]
    adc.set_full_scale_range(FullScaleRange::Within6_144V)
        .unwrap();

    loop {
        // Blink LED 0 to check that everything is actually running.
        // If the LED 0 does not blink, something went wrong.
        led.set_high().unwrap();
        delay.delay_ms(50_u16);
        led.set_low().unwrap();

        // Read voltage in all channels
        let values = [
            block!(adc.read(&mut AdcChannel::SingleA0)).unwrap_or(8091),
            block!(adc.read(&mut AdcChannel::SingleA1)).unwrap_or(8091),
            block!(adc.read(&mut AdcChannel::SingleA2)).unwrap_or(8091),
            block!(adc.read(&mut AdcChannel::SingleA3)).unwrap_or(8091),
        ];

        let mut lines: [heapless::String<32>; 4] = [
            heapless::String::new(),
            heapless::String::new(),
            heapless::String::new(),
            heapless::String::new(),
        ];

        disp.clear();
        for i in 0..values.len() {
            write!(lines[i], "Channel {}: {}", i, values[i]).unwrap();
            Text::with_baseline(
                &lines[i],
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
