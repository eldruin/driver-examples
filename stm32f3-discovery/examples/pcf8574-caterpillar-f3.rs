//! Uses PCF8574 connected to pins PB6 and PB7 of the STM23F3Discovery
//! board to blink LEDs so that they move like a caterpillar.
//! i.e. it outputs 0b0000_0001, then 0b0000_0010 and so on.
//!
//! This example is runs on the STM32F3 Discovery board using I2C1.
//!
//! ```
//! F3  <-> PCF8574
//! GND <-> GND
//! +5V <-> +5V
//! PB7 <-> SDA
//! PB6 <-> SCL
//! ```
//!
//! Run with:
//! `cargo run --example pcf8574-caterpillar-f3 --target thumbv7em-none-eabihf`,

#![deny(unsafe_code)]
#![no_std]
#![no_main]

use cortex_m_rt::entry;
use f3::hal::{delay::Delay, i2c::I2c, prelude::*, stm32f30x};
use panic_semihosting as _;
use pcf857x::{Pcf8574, SlaveAddr};

#[entry]
fn main() -> ! {
    let cp = cortex_m::Peripherals::take().unwrap();
    let dp = stm32f30x::Peripherals::take().unwrap();

    let mut flash = dp.FLASH.constrain();
    let mut rcc = dp.RCC.constrain();
    let clocks = rcc.cfgr.freeze(&mut flash.acr);
    let mut delay = Delay::new(cp.SYST, clocks);

    let mut gpiob = dp.GPIOB.split(&mut rcc.ahb);
    let scl = gpiob.pb6.into_af4(&mut gpiob.moder, &mut gpiob.afrl);
    let sda = gpiob.pb7.into_af4(&mut gpiob.moder, &mut gpiob.afrl);

    let i2c = I2c::i2c1(dp.I2C1, (scl, sda), 400.khz(), clocks, &mut rcc.apb1);
    let mut expander = Pcf8574::new(i2c, SlaveAddr::default());

    let mut output_status = OutputStatus::new();

    loop {
        if let Some(status) = output_status.next() {
            expander.set(status).unwrap();
            delay.delay_ms(100_u16);
        }
    }
}

enum Direction {
    Up,
    Down,
}

struct OutputStatus {
    status: u8,
    direction: Direction,
}

impl OutputStatus {
    pub fn new() -> Self {
        OutputStatus {
            status: 1,
            direction: Direction::Up,
        }
    }
}

impl Iterator for OutputStatus {
    type Item = u8;

    fn next(&mut self) -> Option<Self::Item> {
        match self.direction {
            Direction::Up => {
                if self.status == 64 {
                    self.direction = Direction::Down;
                }
                self.status <<= 1;
            }
            Direction::Down => {
                if self.status == 2 {
                    self.direction = Direction::Up;
                }
                self.status >>= 1;
            }
        }
        Some(self.status)
    }
}
