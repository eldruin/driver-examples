//! Loop setting a position from 0 to 255 to a MCP41010 digital potentiometer.
//!
//! This example is runs on the STM32F1 "Bluepill" board using SPI1.
//!
//! ```
//! BP   <-> MCP41x
//! GND  <-> VSS
//! 3.3V <-> VDD
//! PA5  <-> CLK
//! PA7  <-> SI
//! PA4  <-> CS
//! ```
//!
//! To use the device as a variable resistor (rheostat configuration) connect
//! PW0 to PA0 and measure the resistence between PA0 and PB0.
//! You should see that the resistence varies from 0 to 10K ohm for an MCP41010.
//! The maximum value will be different depending on the exact model.
//! For example, 0-50Kohm for MCP41050 and 0-100Kohm for MCP41100.
//!
//! Run with:
//! `cargo embed --example mcp41x-bp

#![deny(unsafe_code)]
#![no_std]
#![no_main]

use cortex_m_rt::entry;
use embedded_hal::blocking::delay::DelayMs;
use embedded_hal::digital::v2::OutputPin;
use panic_rtt_target as _;
use rtt_target::{rprintln, rtt_init_print};
use stm32f1xx_hal::{delay::Delay, pac, prelude::*, spi::Spi};

use mcp4x::{Channel, Mcp4x, MODE};

#[entry]
fn main() -> ! {
    rtt_init_print!();
    rprintln!("MCP41010 example");

    let cp = cortex_m::Peripherals::take().unwrap();
    let dp = pac::Peripherals::take().unwrap();

    let mut flash = dp.FLASH.constrain();
    let mut rcc = dp.RCC.constrain();

    let clocks = rcc.cfgr.freeze(&mut flash.acr);

    let mut afio = dp.AFIO.constrain(&mut rcc.apb2);
    let mut gpioa = dp.GPIOA.split(&mut rcc.apb2);

    let mut gpioc = dp.GPIOC.split(&mut rcc.apb2);
    let mut led = gpioc.pc13.into_push_pull_output(&mut gpioc.crh);
    let mut delay = Delay::new(cp.SYST, clocks);

    // SPI configuration
    let sck = gpioa.pa5.into_alternate_push_pull(&mut gpioa.crl);
    let miso = gpioa.pa6;
    let mosi = gpioa.pa7.into_alternate_push_pull(&mut gpioa.crl);
    let mut cs = gpioa.pa4.into_push_pull_output(&mut gpioa.crl);

    let spi = Spi::spi1(
        dp.SPI1,
        (sck, miso, mosi),
        &mut afio.mapr,
        MODE,
        1_u32.mhz(),
        clocks,
        &mut rcc.apb2,
    );

    cs.set_high().unwrap();

    let mut digipot = Mcp4x::new_mcp41x(spi, cs);

    let mut position = 0;
    loop {
        // Blink LED 0 to check that everything is actually running.
        // If the LED 0 does not blink, something went wrong.
        led.set_high().unwrap();
        delay.delay_ms(50_u16);
        led.set_low().unwrap();
        delay.delay_ms(50_u16);

        digipot.set_position(Channel::Ch0, position).unwrap();

        if position == 255 {
            position = 0
        } else {
            position += 1;
        }
    }
}
