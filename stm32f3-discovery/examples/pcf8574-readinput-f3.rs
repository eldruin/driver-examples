//! Uses a PCF8574 connected to pins PB6 and PB7 of the STM23F3Discovery
//! board to read the pins P0-P3 and output the values to the LEDs
//! connected to P4-P7.
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
//! `cargo run --example pcf8574-readinput-f3 --target thumbv7em-none-eabihf`,

#![deny(unsafe_code)]
#![no_std]
#![no_main]

use cortex_m_rt::entry;
use panic_rtt_target as _;
use rtt_target::{rprintln, rtt_init_print};
use stm32f3xx_hal::{self as hal, delay::Delay, pac, prelude::*};

use pcf857x::{Pcf8574, PinFlag, SlaveAddr};

#[entry]
fn main() -> ! {
    rtt_init_print!();
    rprintln!("PCF8574 example");

    let cp = cortex_m::Peripherals::take().unwrap();
    let dp = pac::Peripherals::take().unwrap();

    let mut flash = dp.FLASH.constrain();
    let mut rcc = dp.RCC.constrain();
    let clocks = rcc.cfgr.freeze(&mut flash.acr);
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
        400.kHz().try_into().unwrap(),
        clocks,
        &mut rcc.apb1,
    );
    let mut expander = Pcf8574::new(i2c, SlaveAddr::default());

    loop {
        // instead of havin a busy-wait loop like this one, one could use the INT output
        // of the PCF8574 which notifies of changes on the input pins (see datasheet).

        let _input_mask = PinFlag::P0 | PinFlag::P1 | PinFlag::P2 | PinFlag::P3;
        // This does not work yet due to https://github.com/japaric/stm32f30x-hal/pull/27
        // let input = expander.get(_input_mask).unwrap();
        let input = 0b0000_1010;
        // inputs are set to `1` (see PCF8574 datasheet).
        // The status needs to be kept so we `or` the input mask.
        expander.set(input << 4 | 0b0000_1111).unwrap();
        delay.delay_ms(20_u16);
    }
}
