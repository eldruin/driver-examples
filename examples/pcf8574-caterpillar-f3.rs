//! Uses PCF8574 connected to pins PB6 and PB7 of the STM23F3Discovery
//! board to blink LEDs so that they move like a caterpillar.
//! i.e. it outputs 0b0000_0001, then 0b0000_0010 and so on.

#![deny(unsafe_code)]
#![deny(warnings)]
#![no_std]
#![no_main]

extern crate cortex_m;
use cortex_m_rt::entry;
extern crate f3;
extern crate panic_semihosting;
extern crate pcf857x;

use f3::hal::delay::Delay;
use f3::hal::i2c::I2c;
use f3::hal::prelude::*;
use f3::hal::stm32f30x;
pub use f3::hal::stm32f30x::i2c1;
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
        expander.set(output_status.get_status()).unwrap();
        delay.delay_ms(100_u16);
        output_status.increment();
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

    pub fn increment(&mut self) {
        match self.direction {
            Direction::Up => {
                if self.status == 64 {
                    self.direction = Direction::Down;
                }
                self.status = self.status << 1;
            }
            Direction::Down => {
                if self.status == 2 {
                    self.direction = Direction::Up;
                }
                self.status = self.status >> 1;
            }
        }
    }

    pub fn get_status(&self) -> u8 {
        self.status
    }
}
