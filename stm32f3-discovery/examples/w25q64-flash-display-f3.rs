//! Read JEDEC ID of W25Q64 flash memory device and displays it on an
//! SSD1306 OLED display.
//!
//! This example is runs on the STM32F3 Discovery board using SPI1 and I2C1.
//!
//! ```
//! F3   <-> W25Q64 <-> Display
//! GND  <-> GND    <-> GND
//! +3V  <-> VCC
//! +5V             <-> VDD
//! PA5  <-> CLK
//! PA6  <-> DO
//! PA7  <-> DI
//! PB1  <-> CS
//! PB7              <-> SDA     <-> SDA
//! PB6              <-> SCL     <-> SCL
//! ```
//!
//! Run with:
//! `cargo run --example w25q64-flash-display-f3 --target thumbv7em-none-eabihf`,

#![deny(unsafe_code)]
#![no_std]
#![no_main]

use core::fmt::Write;
use cortex_m_rt::entry;
use embedded_graphics::{
    fonts::{Font6x8, Text},
    pixelcolor::BinaryColor,
    prelude::*,
    style::TextStyleBuilder,
};
use embedded_hal::blocking::delay::DelayMs;
use embedded_hal::digital::v2::OutputPin;
use f3::{
    hal::{
        delay::Delay, flash::FlashExt, gpio::GpioExt, i2c::I2c, rcc::RccExt, spi::Spi, stm32f30x,
        time::U32Ext,
    },
    led::Led,
};
use panic_semihosting as _;
use ssd1306::{prelude::*, Builder, I2CDIBuilder};
use w25::{MODE_0, W25};

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

    let scl = gpiob.pb6.into_af4(&mut gpiob.moder, &mut gpiob.afrl);
    let sda = gpiob.pb7.into_af4(&mut gpiob.moder, &mut gpiob.afrl);

    let i2c = I2c::i2c1(dp.I2C1, (scl, sda), 400.khz(), clocks, &mut rcc.apb1);

    let interface = I2CDIBuilder::new().init(i2c);
    let mut disp: GraphicsMode<_> = Builder::new().connect(interface).into();
    disp.init().unwrap();
    disp.flush().unwrap();

    let text_style = TextStyleBuilder::new(Font6x8)
        .text_color(BinaryColor::On)
        .build();

    // SPI configuration
    let sck = gpioa.pa5.into_af5(&mut gpioa.moder, &mut gpioa.afrl);
    let miso = gpioa.pa6.into_af5(&mut gpioa.moder, &mut gpioa.afrl);
    let mosi = gpioa.pa7.into_af5(&mut gpioa.moder, &mut gpioa.afrl);

    let spi = Spi::spi1(
        dp.SPI1,
        (sck, miso, mosi),
        MODE_0,
        1.mhz(),
        clocks,
        &mut rcc.apb2,
    );

    let mut chip_select = gpiob
        .pb1
        .into_push_pull_output(&mut gpiob.moder, &mut gpiob.otyper);

    chip_select.set_high().unwrap();

    let mut flash = W25::new_w25q64(spi, chip_select);
    loop {
        // Blink LED 0 to check that everything is actually running.
        // If the LED 0 does not blink, something went wrong.
        led.on();
        delay.delay_ms(50_u16);
        led.off();

        let id = flash.get_jedec_id().unwrap_or([255; 3]);

        let mut msg: heapless::String<heapless::consts::U64> = heapless::String::new();

        write!(msg, "JEDEC ID: {} {} {}", id[0], id[1], id[2]).unwrap();
        disp.clear();
        Text::new(&msg, Point::zero())
            .into_styled(text_style)
            .draw(&mut disp)
            .unwrap();
        disp.flush().unwrap();
    }
}
