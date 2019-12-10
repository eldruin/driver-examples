//! Demonstration of controlling two RGB LEDs to display a rainbow and
//! moving 5 servos simultaneously.
//! 
//! You can see a video of this program running here:
//! https://blog.eldruin.com/pca9685-pwm-led-servo-controller-driver-in-rust/
//!
//! This example is runs on the STM32F103 "Bluepill" board using I2C1.
//!
//! ```
//! BP    <-> Pca9685
//! GND   <-> GND
//! +3.3V <-> VCC
//! PB8   <-> SCLK
//! PB9   <-> SDIO
//! GND   <-> OE
//!           V+      <-> +5V
//! ```
//!
//! Run with:
//! `cargo run --example pca9685-pwm-rgb-led-servos-bp`,

#![deny(unsafe_code)]
#![no_std]
#![no_main]

use cortex_m_rt::entry;
use panic_semihosting as _;
use pwm_pca9685::{Pca9685, SlaveAddr};
use stm32f1xx_hal::{
    delay::Delay,
    i2c::{BlockingI2c, DutyCycle, Mode},
    pac,
    prelude::*,
};

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
    let sda = gpiob.pb9.into_alternate_open_drain(&mut gpiob.crh);
    let mut delay = Delay::new(cp.SYST, clocks);

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

    let mut pwm = Pca9685::new(i2c, SlaveAddr::default());
    pwm.enable().unwrap();
    pwm.set_prescale(100).unwrap();
    
    let mut rainbow = Rainbow::new(0);
    let mut servos = [
        Servo::new(Servo::MIN),
        Servo::new(Servo::MIN+40),
        Servo::new(Servo::MIN+80),
        Servo::new(Servo::MIN+120),
        Servo::new(Servo::MIN+160),
    ];
    let mut values = [0; 16];
    loop {
        if let Some((r, g, b)) = rainbow.next() {
            const MAX: u16 = 4080 >> 5;
            delay.delay_ms(1_u16);
            let (r, g, b) = (r >> 5, g >> 5, b >> 5); // make LEDs less bright
            values[0] = r;
            values[1] = g;
            values[2] = b;
            values[3] = MAX - r;
            values[4] = MAX - g;
            values[5] = MAX - b;
            for (i, servo) in servos.iter_mut().enumerate() {
                if let Some(v) = servo.next() {
                    values[i + 10] = v;
                }
            }
            pwm.set_all_on_off(&[0; 16], &values).unwrap();
            // you can also set individual channels with something like:
            // pwm.set_channel_on_off(Channel::C0, 0, 2047).unwrap();
        }
    }
}

/// RGB rainbow generator
struct Rainbow {
    hue: u16,
}

impl Rainbow {
    fn new(hue: u16) -> Self {
        Rainbow { hue }
    }
}

impl Iterator for Rainbow {
    type Item = (u16, u16, u16);

    fn next(&mut self) -> Option<Self::Item> {
        // See HSV to RGB conversion: https://en.wikipedia.org/wiki/HSL_and_HSV
        // To avoid floating point calculations and ensure smooth transitions
        // the value range is limited to [0-4080] as 4080 = 60*68.
        self.hue = (self.hue + 1) % 361;
        match self.hue {
            0..=59 => Some((4080, self.hue * 68, 0)),
            60..=119 => Some(((120 - self.hue) * 68, 4080, 0)),
            120..=179 => Some((0, 4080, (self.hue - 120) * 68)),
            180..=239 => Some((0, (240 - self.hue) * 68, 4080)),
            240..=299 => Some(((self.hue - 240) * 68, 0, 4080)),
            300..=360 => Some((4080, 0, (360 - self.hue) * 68)),
            _ => None,
        }
    }
}

struct Servo {
    current: u16,
    factor: i16,
}

impl Servo {
    // You need to tweak these min/max values for your servos as these may vary.
    // Be careful when doing this. Incorrect values can permanently damage your servos.
    const MIN: u16 = 132; // minimum pulse length
    const MAX: u16 = 608; // maximum pulse length

    fn new(offset: u16) -> Self {
        Servo {
            current: offset,
            factor: 2,
        }
    }
}

impl Iterator for Servo {
    type Item = u16;

    fn next(&mut self) -> Option<Self::Item> {
        if self.current >= Self::MAX {
            self.factor = -2;
        } else if self.current <= Self::MIN {
            self.factor = 2;
        }
        self.current = ((self.current as i16) + self.factor) as u16;
        Some(self.current)
    }
}
