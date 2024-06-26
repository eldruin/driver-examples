//! Control a servo connected to channel 0 and one connected to channel 1.
//!
//! Make the servo at channel 0 turn clockwise, then counter-clockwise and
//! the servo at channel 1 does the opposite.
//!
//! You can see a video of this device running here:
//! https://blog.eldruin.com/pca9685-pwm-led-servo-controller-driver-in-rust/
//!
//! This example is runs on the STM32F3 Discovery board using I2C1.
//!
//! ```
//! F3  <-> PCA9685
//! GND <-> GND
//! VCC <-> +5V
//! PB7 <-> SDA
//! PB6 <-> SCL
//! ```
//! Run with:
//! `cargo run --example pca9685-servos-f3 --target thumbv7em-none-eabihf`,
//! currently only works on nightly.

#![deny(unsafe_code)]
#![no_std]
#![no_main]

use cortex_m_rt::entry;
use panic_rtt_target as _;
use rtt_target::{rprintln, rtt_init_print};
use stm32f3xx_hal::{self as hal, delay::Delay, pac, prelude::*};

use pwm_pca9685::{Address, Channel, Pca9685};

#[entry]
fn main() -> ! {
    rtt_init_print!();
    rprintln!("PCA9685 example");

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

    let mut pwm = Pca9685::new(i2c, Address::default()).unwrap();
    // This results in about 60 Hz, which is the frequency at which servos operate.
    pwm.set_prescale(100).unwrap();
    pwm.enable().unwrap();
    // Turn all channels on at time "0".
    pwm.set_channel_on(Channel::All, 0).unwrap();

    // You need to tweak these min/max values for your servos as these may vary.
    let servo_min = 130; // minimum pulse length (out of 4096)
    let servo_max = 610; // maximum pulse length (out of 4096)
    let mut current = servo_min;
    let mut factor: i16 = 1;
    loop {
        // Blink LED 0 (really fast, it will seem to be on the whole time)
        // to check that everything is actually running.
        // If the LED 0 is off, something went wrong.
        led.set_high().unwrap();
        delay.delay_ms(1_u16);
        led.set_low().unwrap();
        delay.delay_ms(1_u16);

        pwm.set_channel_off(Channel::C0, current).unwrap();
        pwm.set_channel_off(Channel::C1, servo_min + (servo_max - current))
            .unwrap();

        if current == servo_max {
            factor = -1;
        } else if current == servo_min {
            factor = 1;
        }
        current = (current as i16 + factor) as u16;
    }
}
