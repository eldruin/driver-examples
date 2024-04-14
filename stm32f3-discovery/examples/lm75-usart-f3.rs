//! Continuously read the temperature every second with the LM75 and
//! transmit it per USART.
//!
//! When running you should be able to see the temperature readings in your
//! serial communication program.
//!
//! This example is runs on the STM32F3 Discovery board using I2C1 and USART1.
//!
//! To setup the serial communication, have a look at the discovery book:
//! https://rust-embedded.github.io/discovery/10-serial-communication/index.html
//!
//! ```
//! F3  <-> LM75
//! GND <-> GND
//! VCC <-> +5V
//! PB7 <-> SDA
//! PB6 <-> SCL
//!
//! F3   <-> Serial device
//! GND  <-> GND
//! PA9  <-> TX
//! PA10 <-> RX
//! ```
//!
//! Run with:
//! `cargo run --example lm75-usart-f3 --target thumbv7em-none-eabihf`,

#![no_std]
#![no_main]

use core::fmt::Write;
use cortex_m_rt::entry;
use panic_rtt_target as _;
use rtt_target::{rprintln, rtt_init_print};
use stm32f3xx_hal::{self as hal, delay::Delay, pac, prelude::*, serial::Serial};

use lm75::{Address, Lm75};

#[entry]
fn main() -> ! {
    rtt_init_print!();
    rprintln!("LM75 example");

    let cp = cortex_m::Peripherals::take().unwrap();
    let dp = pac::Peripherals::take().unwrap();

    let mut flash = dp.FLASH.constrain();
    let mut rcc = dp.RCC.constrain();
    let clocks = rcc.cfgr.freeze(&mut flash.acr);

    let mut delay = Delay::new(cp.SYST, clocks);

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

    let mut lm75 = Lm75::new(i2c, Address::default());

    loop {
        delay.delay_ms(1000_u16);

        let temp = lm75.read_temperature().unwrap();

        // transform number to string
        let mut buffer: heapless::String<1616> = heapless::String::new();
        write!(buffer, "{} ", temp).unwrap();

        // send buffer
        serial.bwrite_all(&buffer.into_bytes()).unwrap();
        serial.bflush().unwrap();
    }
}
