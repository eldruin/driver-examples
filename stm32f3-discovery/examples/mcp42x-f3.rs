//! Loop setting a position from 0 to 255 to the channel 0 of a MCP42010
//! digital potentiometer and its inverse to channel 1.
//!
//! This example is runs on the STM32F3 Discovery board using SPI1.
//!
//! ```
//! F3   <-> MCP42x
//! GND  <-> VSS
//! 3.3V <-> VDD
//! PA5  <-> CLK
//! PA7  <-> SI
//! PB5  <-> CS
//! ```
//!
//! To use the device as a variable resistor (rheostat configuration) connect
//! PW0 to PA0 and measure the resistence between PA0 and PB0.
//! You should see that the resistence varies from 0 to 10K ohm for an MCP42010.
//! The maximum value will be different depending on the exact model.
//! For example, 0-50Kohm for MCP42050 and 0-100Kohm for MCP42100.
//!
//! Run with:
//! `cargo run --example mcp42x-f3 --target thumbv7em-none-eabihf`,

#![deny(unsafe_code)]
#![no_std]
#![no_main]

use panic_semihosting as _;

use cortex_m_rt::entry;
use embedded_hal::blocking::delay::DelayMs;
use embedded_hal::digital::v2::OutputPin;
use f3::{
    hal::{
        delay::Delay, flash::FlashExt, gpio::GpioExt, rcc::RccExt, spi::Spi, stm32f30x,
        time::U32Ext,
    },
    led::Led,
};
use mcp4x::{Channel, Mcp4x, MODE};

#[entry]
fn main() -> ! {
    let cp = cortex_m::Peripherals::take().unwrap();
    let dp = stm32f30x::Peripherals::take().unwrap();

    let mut flash = dp.FLASH.constrain();
    let mut rcc = dp.RCC.constrain();
    let mut gpioe = dp.GPIOE.split(&mut rcc.ahb);
    let clocks = rcc.cfgr.freeze(&mut flash.acr);

    let mut led: Led = gpioe
        .pe9
        .into_push_pull_output(&mut gpioe.moder, &mut gpioe.otyper)
        .into();
    let mut delay = Delay::new(cp.SYST, clocks);

    let mut gpiob = dp.GPIOB.split(&mut rcc.ahb);
    let mut gpioa = dp.GPIOA.split(&mut rcc.ahb);

    // SPI configuration
    let sck = gpioa.pa5.into_af5(&mut gpioa.moder, &mut gpioa.afrl);
    let miso = gpioa.pa6.into_af5(&mut gpioa.moder, &mut gpioa.afrl);
    let mosi = gpioa.pa7.into_af5(&mut gpioa.moder, &mut gpioa.afrl);

    let spi = Spi::spi1(
        dp.SPI1,
        (sck, miso, mosi),
        MODE,
        1.mhz(),
        clocks,
        &mut rcc.apb2,
    );

    let mut chip_select = gpiob
        .pb5
        .into_push_pull_output(&mut gpiob.moder, &mut gpiob.otyper);

    chip_select.set_high().unwrap();

    let mut digipot = Mcp4x::new_mcp42x(spi, chip_select);

    let mut position = 0;
    loop {
        // Blink LED 0 to check that everything is actually running.
        // If the LED 0 does not blink, something went wrong.
        led.on();
        delay.delay_ms(50_u16);
        led.off();
        delay.delay_ms(50_u16);

        digipot.set_position(Channel::Ch0, position).unwrap();
        digipot.set_position(Channel::Ch1, 255 - position).unwrap();

        if position == 255 {
            position = 0
        } else {
            position += 1;
        }
    }
}
