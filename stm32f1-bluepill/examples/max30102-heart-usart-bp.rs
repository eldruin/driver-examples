//! Continuously read the heart data and send it through USART.
//!
//! This example is runs on the STM32F103 "Bluepill" board using I2C1 and USART1.
//!
//! To setup the serial communication, have a look at the discovery book:
//! https://rust-embedded.github.io/discovery/10-serial-communication/index.html
//!
//! ```
//! BP   <-> MAX30102 <-> Serial
//! GND  <-> GND      <-> GND
//! 3.3V <-> VCC      <-> VDD
//! PB6               <-> RX
//! PB8  <-> SCL
//! PB9  <-> SDA
//! ```
//!
//! Run with:
//! `cargo embed --example max30102-heart-usart-bp --release`,

#![deny(unsafe_code)]
#![no_std]
#![no_main]

use core::fmt::Write;
use cortex_m_rt::entry;
use max3010x::{Led, LedPulseWidth, Max3010x, SampleAveraging, SamplingRate};
use panic_rtt_target as _;
use rtt_target::{rprintln, rtt_init_print};
use stm32f1xx_hal::{
    delay::Delay,
    i2c::{BlockingI2c, DutyCycle, Mode},
    pac,
    prelude::*,
    serial,
};

#[entry]
fn main() -> ! {
    rtt_init_print!();
    rprintln!("MAX30102 example");

    let cp = cortex_m::Peripherals::take().unwrap();
    let dp = pac::Peripherals::take().unwrap();

    let mut flash = dp.FLASH.constrain();
    let rcc = dp.RCC.constrain();
    let clocks = rcc
        .cfgr
        .use_hse(8.mhz())
        .sysclk(72.mhz())
        .pclk1(36.mhz())
        .freeze(&mut flash.acr);
    let mut afio = dp.AFIO.constrain();
    let mut gpiob = dp.GPIOB.split();
    let mut delay = Delay::new(cp.SYST, clocks);

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

    let tx = gpiob.pb6.into_alternate_push_pull(&mut gpiob.crl);
    let rx = gpiob.pb7;
    let serial = serial::Serial::usart1(
        dp.USART1,
        (tx, rx),
        &mut afio.mapr,
        serial::Config::default().baudrate(9600.bps()),
        clocks,
    );
    let (mut tx, _rx) = serial.split();

    let mut max30102 = Max3010x::new_max30102(i2c);
    max30102.reset().unwrap();
    delay.delay_ms(100_u8);

    let mut max30102 = max30102.into_heart_rate().unwrap();

    max30102.enable_fifo_rollover().unwrap();
    max30102.set_pulse_amplitude(Led::All, 15).unwrap();
    max30102.set_sample_averaging(SampleAveraging::Sa8).unwrap();
    max30102.set_sampling_rate(SamplingRate::Sps100).unwrap();
    max30102.set_pulse_width(LedPulseWidth::Pw411).unwrap();

    max30102.clear_fifo().unwrap();

    writeln!(tx, "hr\r").unwrap();
    loop {
        delay.delay_ms(100_u8);
        let mut data = [0; 16];
        let read = max30102.read_fifo(&mut data).unwrap_or(0);
        for i in 0..read.into() {
            writeln!(tx, "{}\r", data[i],).unwrap();
        }
    }
}
