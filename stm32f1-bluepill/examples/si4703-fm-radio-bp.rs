//! Automatically seek for an FM radio channel every 5 seconds using
//! an Si4703 FM radio receiver (turner)
//!
//! This example is runs on the STM32F103 "Bluepill" board using I2C1.
//!
//! ```
//! BP   <-> Si4703
//! GND  <-> GND
//! +5V  <-> VCC
//! PB8  <-> SCLK
//! PB9  <-> SDIO
//! PB7  <-> RST
//! PB6  <-> GPIO2
//! ```
//!
//! Run with:
//! `cargo run --example si4703-fm-radio-bp`,

#![deny(unsafe_code)]
#![no_std]
#![no_main]

use cortex_m_rt::entry;
use embedded_hal::digital::v2::OutputPin;
use panic_halt as _;
use nb::block;
use stm32f1xx_hal::{
    delay::Delay,
    i2c::{BlockingI2c, DutyCycle, Mode},
    pac,
    prelude::*,
};
use si470x::{reset as reset_si470x, Si470x, SeekMode,SeekDirection, DeEmphasis};


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
    let mut sda = gpiob.pb9.into_alternate_open_drain(&mut gpiob.crh);
    let mut rst = gpiob.pb7.into_push_pull_output(&mut gpiob.crl);
    let mut stcint = gpiob.pb6;
    let mut gpioc = dp.GPIOC.split(&mut rcc.apb2);
    let mut led = gpioc.pc13.into_push_pull_output(&mut gpioc.crh);
    let mut delay = Delay::new(cp.SYST, clocks);

    reset_si470x(&mut rst, &mut sda, &mut delay).unwrap();
    let i2c = BlockingI2c::i2c1(
        dp.I2C1,
        (scl, sda),
        &mut afio.mapr,
        Mode::Fast {
            frequency: 400_000,
            duty_cycle: DutyCycle::Ratio2to1,
        },
        clocks,
        &mut rcc.apb1,
        1000,
        10,
        1000,
        1000,
    );

    let mut radio = Si470x::new_si4703(i2c);
    radio.enable_oscillator().unwrap();
    delay.delay_ms(500_u16);
    radio.enable().unwrap();
    delay.delay_ms(110_u16);

    radio.set_volume(1).unwrap();
    radio.set_deemphasis(DeEmphasis::Us50).unwrap();
    radio.configure_seek(SeekMode::Wrap, SeekDirection::Up).unwrap();
    radio.unmute().unwrap();
    loop {
        // Blink LED 0 every time a new seek is started
        // to check that everything is actually running.
        led.set_low().unwrap();
        delay.delay_ms(50_u16);
        led.set_high().unwrap();

        block!(radio.seek_with_stc_int_pin(&mut stcint)).unwrap();
        delay.delay_ms(5000_u16);
   }
}