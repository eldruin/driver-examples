//! Stores some data on an AT24C256C EEPROM.
//! Then reads it again and if it matches, blinks LED 0.
//!
//! Introductory blog post here:
//! https://blog.eldruin.com/24x-serial-eeprom-driver-in-rust/
//!
//! This example is runs on the STM32F1 "BluePill" board using I2C1.
//!
//! ```
//! BP  <-> AT24C256
//! GND <-> GND
//! +5V <-> +5V
//! PB8 <-> SCL
//! PB9 <-> SDA
//! ```
//!
//! Run with:
//! `cargo embed --example at24c256-eeprom-bp`,

#![deny(unsafe_code)]
#![no_std]
#![no_main]

use cortex_m_rt::entry;
use eeprom24x::{Eeprom24x, SlaveAddr};
use embedded_hal::digital::v2::OutputPin;
use panic_rtt_target as _;
use rtt_target::{rprintln, rtt_init_print};
use stm32f1xx_hal::{
    delay::Delay,
    i2c::{BlockingI2c, DutyCycle, Mode},
    pac,
    prelude::*,
};

#[entry]
fn main() -> ! {
    rtt_init_print!();
    rprintln!("AT24C256 example");
    let cp = cortex_m::Peripherals::take().unwrap();
    let dp = pac::Peripherals::take().unwrap();

    let mut flash = dp.FLASH.constrain();
    let mut rcc = dp.RCC.constrain();

    let clocks = rcc.cfgr.freeze(&mut flash.acr);

    let mut afio = dp.AFIO.constrain(&mut rcc.apb2);

    let mut gpiob = dp.GPIOB.split(&mut rcc.apb2);

    let scl = gpiob.pb8.into_alternate_open_drain(&mut gpiob.crh);
    let sda = gpiob.pb9.into_alternate_open_drain(&mut gpiob.crh);

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

    let mut eeprom = Eeprom24x::new_24x256(i2c, SlaveAddr::Alternative(true, true, true));
    let memory_address = 0x01;
    eeprom
        .write_page(memory_address, &[0xAB, 0xCD, 0xEF, 0x12])
        .unwrap();

    // wait maximum time necessary for write
    delay.delay_ms(5_u16);
    loop {
        let mut data = [0; 4];
        eeprom.read_data(memory_address, &mut data).unwrap();
        if data == [0xAB, 0xCD, 0xEF, 0x12] {
            led.set_high().unwrap();
            delay.delay_ms(250_u16);
            led.set_low().unwrap();
            delay.delay_ms(250_u16);
        }
    }
}
