//! This example continuously reads the samples from a MAX30102 heart-rate
//! and pulse oximeter (SpO2) sensor in heart-rate mode and displays the
//! values in an SSD1306 OLED display.
//!
//! This example is runs on the STM32F3 Discovery board using I2C1.
//!
//! ```
//! F3    <-> MAX30102 <-> Display
//! GND   <-> GND      <-> GND
//! +3.3V <-> +3.3V
//! +5V   <-> +5V      <-> +5V
//! PB7   <-> SDA      <-> SDA
//! PB6   <-> SCL      <-> SCL
//! ```
//! Run with:
//! `cargo run --example max30102-display-f3 --target thumbv7em-none-eabihf`

#![deny(unsafe_code)]
#![no_std]
#![no_main]

use core::convert::TryInto;
use core::fmt::Write;
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
use stm32f3xx_hal::{self as hal, delay::Delay, pac, prelude::*};

use max3010x::{Led as MaxLed, Max3010x};

#[entry]
fn main() -> ! {
    rtt_init_print!();
    rprintln!("MAX30102 example");

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

    let mut max30102 = Max3010x::new_max30102(manager.acquire());

    led.set_high().unwrap();
    delay.delay_ms(50_u16);
    led.set_low().unwrap();
    delay.delay_ms(50_u16);
    max30102.reset().unwrap();

    let mut max30102 = max30102.into_heart_rate().unwrap();
    max30102.set_pulse_amplitude(MaxLed::All, 15).unwrap();
    max30102.enable_fifo_rollover().unwrap();
    loop {
        // Blink LED 0 to check that everything is actually running.
        // If the LED 0 is off, something went wrong.
        led.set_high().unwrap();
        delay.delay_ms(50_u16);
        led.set_low().unwrap();
        delay.delay_ms(50_u16);

        let mut buffer: heapless::String<64> = heapless::String::new();

        let mut data = [0; 3];
        let _read = max30102.read_fifo(&mut data).unwrap_or(0xFF);

        write!(buffer, "{}, {}, {}", data[0], data[1], data[2]).unwrap();
        disp.clear();
        Text::with_baseline(&buffer, Point::zero(), text_style, Baseline::Top)
            .draw(&mut disp)
            .unwrap();

        disp.flush().unwrap();
    }
}
