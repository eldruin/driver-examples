//! Uses PCF8574 I/O expander to blink LEDs so that they move like a caterpillar.
//! i.e. it outputs 0b0000_0001, then 0b0000_0010 and so on.
//!
//! This example is runs on the STM32F1 "BluePill" board using I2C1.
//!
//! ```
//! BP  <-> PCF8574
//! GND <-> GND
//! +5V <-> +5V
//! PB8 <-> SCL
//! PB9 <-> SDA
//! ```
//!
//! Run with:
//! `cargo embed --example pcf8574-caterpillar-bp`,

#![deny(unsafe_code)]
#![no_std]
#![no_main]

use cortex_m_rt::entry;
use panic_rtt_target as _;
use pcf857x::{Pcf8574, SlaveAddr};
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
    rprintln!("PCF8574 example");
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

    let mut delay = Delay::new(cp.SYST, clocks);
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
