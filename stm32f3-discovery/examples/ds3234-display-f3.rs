//! Stores the date and time on a DS3234 real-time clock (RTC).
//! Then reads the date and time repeatedly and prints it to an
//! SSD1306 OLED display.
//!
//! This example is runs on the STM32F3 Discovery board using SPI1 and I2C1.
//!
//! ```
//! F3    <-> DS3234  <-> Display
//! GND   <-> GND     <-> GND
//! +3.3V <-> +3.3V
//! +5V               <-> +5V
//! PA5   <-> CLK
//! PA6   <-> DO
//! PA7   <-> DI
//! PB1   <-> CS
//! PB7               <-> SDA
//! PB6               <-> SCL
//! ```
//!
//! Run with:
//! `cargo run --example ds3234-display-f3`,

#![deny(unsafe_code)]
#![no_std]
#![no_main]

use panic_semihosting as _;

use cortex_m_rt::entry;
use embedded_hal::blocking::delay::DelayMs;
use embedded_hal::digital::v2::OutputPin;
use embedded_hal::spi::MODE_1;
use f3::{
    hal::{
        delay::Delay, flash::FlashExt, gpio::GpioExt, i2c::I2c, rcc::RccExt, spi::Spi, stm32f30x,
        time::U32Ext,
    },
    led::Led,
};

use core::fmt::Write;
use embedded_graphics::{
    fonts::{Font6x8, Text},
    pixelcolor::BinaryColor,
    prelude::*,
    style::TextStyleBuilder,
};
use ssd1306::{prelude::*, Builder, I2CDIBuilder};

use ds323x::{Ds323x, NaiveDate};
use rtcc::Rtcc;

#[entry]
fn main() -> ! {
    let cp = cortex_m::Peripherals::take().unwrap();
    let dp = stm32f30x::Peripherals::take().unwrap();

    let mut flash = dp.FLASH.constrain();
    let mut rcc = dp.RCC.constrain();
    let clocks = rcc.cfgr.freeze(&mut flash.acr);
    let mut delay = Delay::new(cp.SYST, clocks);

    let mut gpioe = dp.GPIOE.split(&mut rcc.ahb);
    let mut led: Led = gpioe
        .pe9
        .into_push_pull_output(&mut gpioe.moder, &mut gpioe.otyper)
        .into();

    let mut gpioa = dp.GPIOA.split(&mut rcc.ahb);
    let mut gpiob = dp.GPIOB.split(&mut rcc.ahb);

    // SPI configuration
    let sck = gpioa.pa5.into_af5(&mut gpioa.moder, &mut gpioa.afrl);
    let miso = gpioa.pa6.into_af5(&mut gpioa.moder, &mut gpioa.afrl);
    let mosi = gpioa.pa7.into_af5(&mut gpioa.moder, &mut gpioa.afrl);
    led.on();
    delay.delay_ms(500_u16);
    led.off();
    delay.delay_ms(500_u16);

    let scl = gpiob.pb6.into_af4(&mut gpiob.moder, &mut gpiob.afrl);
    let sda = gpiob.pb7.into_af4(&mut gpiob.moder, &mut gpiob.afrl);

    let i2c = I2c::i2c1(dp.I2C1, (scl, sda), 100.khz(), clocks, &mut rcc.apb1);
    let interface = I2CDIBuilder::new().init(i2c);
    let mut disp: GraphicsMode<_> = Builder::new().connect(interface).into();
    disp.init().unwrap();
    disp.flush().unwrap();

    let text_style = TextStyleBuilder::new(Font6x8)
        .text_color(BinaryColor::On)
        .build();

    let spi = Spi::spi1(
        dp.SPI1,
        (sck, miso, mosi),
        MODE_1,
        1.mhz(),
        clocks,
        &mut rcc.apb2,
    );

    let mut chip_select = gpiob
        .pb1
        .into_push_pull_output(&mut gpiob.moder, &mut gpiob.otyper);

    chip_select.set_high().unwrap();

    let mut rtc = Ds323x::new_ds3234(spi, chip_select);
    let begin = NaiveDate::from_ymd(2020, 5, 2).and_hms(13, 50, 23);
    rtc.disable().unwrap();
    rtc.set_datetime(&begin).unwrap();
    rtc.enable().unwrap();
    loop {
        let now = rtc.get_datetime().unwrap();
        let mut line: heapless::String<heapless::consts::U32> = heapless::String::new();

        write!(line, "{}", now).unwrap();
        disp.clear();
        Text::new(&line, Point::zero())
            .into_styled(text_style)
            .draw(&mut disp)
            .unwrap();
        disp.flush().unwrap();
    }
}
