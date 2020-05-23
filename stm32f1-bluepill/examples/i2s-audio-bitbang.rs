//! This example plays a scala of tones through I2S by bitbanging the
//! protocol out of some pins connected to an I2S receiver like
//! MAX98357A, MAX98357B, PCM5102A, etc.
//! These can be then connected to a speaker.
//!
//! This example runs on the STM32F103 "Bluepill" board using 3 GPIO and
//! generates approximately an 8kHz digital audio signal.
//! Note when using other I2S receiver chips that they must support this sampling
//! frequency.
//!
//! ```
//! BP    <-> MAX98357A
//! GND   <-> GND
//! +3.3V <-> VCC
//! +3.3V <-> SD (Shutdown)
//! PB7   <-> LRC (Word select)
//! PB6   <-> DIN (Data)
//! PB10  <-> BCLK (Clock)
//! ```
//!
//! ```
//! BP    <-> PCM5102A (Standard I2S mode)
//! GND   <-> GND
//! +3.3V <-> +3.3V
//! +5V   <-> VCC
//! PB7   <-> LCK (Word select)
//! PB6   <-> DIN (Data)
//! PB10  <-> BCLK (Clock)
//! GND   <-> SCL (Master clock is internally generated)
//! +3.3V <-> FMT
//! +3.3V <-> XMT
//! ```
//! FLT and DMP can be left unconnected.
//!
//! It is important to run this in release mode, otherwise the generation will be too slow.
//!
//! Run with:
//! `cargo run --example i2s --release`

#![deny(unsafe_code)]
#![no_std]
#![no_main]

use bitbang_hal::i2s::{I2s, Mode as I2sMode};
use cortex_m_rt::entry;
use libm::{roundf, sinf};
use panic_semihosting as _;
use stm32f1xx_hal::{
    pac, timer::Timer,
    prelude::*,
};

const PI: f32 = 3.14159;

#[entry]
fn main() -> ! {
    let dp = pac::Peripherals::take().unwrap();

    let mut flash = dp.FLASH.constrain();
    let mut rcc = dp.RCC.constrain();

    // Max all clocks to ensure enough performance for bitbanging
    let clocks = rcc
        .cfgr
        .use_hse(8.mhz())
        .sysclk(72.mhz())
        .pclk1(36.mhz())
        .freeze(&mut flash.acr);

    // We need to generate a very high frequency signal.
    // The minimum sampling frequency of MAX98357A/B and PCM5102A is 8kHz.
    // We are sending 16-bit audio times 2 channels so we need an output clock frequency of:
    // 8 kHz * 16 * 2 = 256 kHz
    //
    // Additionally, for bitbanging we need a timer that fires at double that frequency to
    // control the signals during the clock low and high phase.
    // This results in 512 kHz.
    let timer = Timer::tim3(dp.TIM3, 512.khz(), clocks, &mut rcc.apb1);
    let mut gpiob = dp.GPIOB.split(&mut rcc.apb2);

    let ws = gpiob.pb7.into_push_pull_output(&mut gpiob.crl);
    let sd = gpiob.pb6.into_push_pull_output(&mut gpiob.crl);
    let clk = gpiob.pb10.into_push_pull_output(&mut gpiob.crh);

    // Use standard I2S mode.
    // Can be changed to left-adjusted for MAX98357B or PCM5102 in left-adjusted mode.
    let mut i2s = I2s::new(I2sMode::I2s, sd, ws, clk, timer);

    // Classic sine wave table in the 16-bit signed integer value range
    let mut sine_table = [0; 200];
    for i in 0..sine_table.len() {
        sine_table[i] =
            roundf(32365.0 * sinf(2.0 * PI * (i as f32) / (sine_table.len() as f32))) as i16
    }
    loop {
        // this works a bit like a numerically controlled oscillator
        // increasing, then decreasing pitch.
        for pitch in (10..20).chain((11..20).rev()) {
            // play each tone for a short while
            for _ in 0..50 {
                for i in (0..sine_table.len()).step_by(pitch) {
                    i2s.try_write(&[sine_table[i]], &[sine_table[i]]).unwrap();
                }
            }
        }
    }
}
