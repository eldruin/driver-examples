//! Continuously read the acceleration with a KXCJ9-1018 and
//! transmit it per USART. (trivially adaptable to use an KXCJ9-1008).
//!
//! When running you should be able to see the acceleration readings in your
//! serial communication program.
//!
//! Introductory blog post here:
//! https://blog.eldruin.com/kxcj9-kxcjb-tri-axis-mems-accelerator-driver-in-rust/
//!
//! This example is runs on the STM32F3 Discovery board using I2C1 and USART1.
//!
//! To setup the serial communication, have a look at the discovery book:
//! https://rust-embedded.github.io/discovery/10-serial-communication/index.html
//!
//! ```
//! F3    <-> KXCJ9
//! GND   <-> GND
//! +3.3V <-> VCC
//! PB7   <-> SDA
//! PB6   <-> SCL
//!
//! F3   <-> Serial device
//! GND  <-> GND
//! PA9  <-> TX
//! PA10 <-> RX
//! ```
//!
//! Beware that the KXCJ9 runs on 3.3V but PB6 and PB7 run on 5V level
//! so make sure to put a logic level shifter in between.
//!
//! Run with:
//! `cargo run --example kxcj9-usart-f3 --target thumbv7em-none-eabihf`,

#![no_std]
#![no_main]

use core::convert::TryInto;
use cortex_m_rt::entry;
use panic_rtt_target as _;
use rtt_target::{rprintln, rtt_init_print};
use stm32f3xx_hal::{self as hal, pac, prelude::*, serial::Serial};

use kxcj9::{Kxcj9, SlaveAddr};

use core::fmt::Write;
#[entry]
fn main() -> ! {
    rtt_init_print!();
    rprintln!("KXCJ9 example");

    let dp = pac::Peripherals::take().unwrap();

    let mut flash = dp.FLASH.constrain();
    let mut rcc = dp.RCC.constrain();
    let clocks = rcc.cfgr.freeze(&mut flash.acr);

    let mut gpioa = dp.GPIOA.split(&mut rcc.ahb);
    let pins = (
        gpioa
            .pa9
            .into_af7_push_pull(&mut gpioa.moder, &mut gpioa.otyper, &mut gpioa.afrh),
        gpioa
            .pa10
            .into_af7_push_pull(&mut gpioa.moder, &mut gpioa.otyper, &mut gpioa.afrh),
    );
    let mut serial = Serial::new(dp.USART1, pins, 115_200.Bd(), clocks, &mut rcc.apb2);

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

    let mut accelerometer = Kxcj9::new_kxcj9_1018(i2c, SlaveAddr::default());
    accelerometer.enable().unwrap();

    loop {
        let accel = accelerometer.read().unwrap();

        // transform numbers to string
        let mut buffer: heapless::String<1616> = heapless::String::new();
        write!(buffer, "{},{},{} ", accel.x, accel.y, accel.z).unwrap();

        serial.bwrite_all(&buffer.into_bytes()).unwrap();
        serial.bflush().unwrap();
    }
}
