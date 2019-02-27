//! This example continuously reads the samples from a MAX30102 heart-rate
//! and pulse oximeter (SpO2) sensor in heart-rate mode and displays the
//! values in an SSD1306 OLED display.
//!
//! This example is runs on the STM32F3 Discovery board using I2C1.
//!
//! ```
//! F3    <-> MAX30102 <-> Display
//! GND   <-> GND      <-> GND
//! +3.3V <-> +3.3V
//! +5V   <-> +5V      <-> +5V
//! PB7   <-> SDA      <-> SDA
//! PB6   <-> SCL      <-> SCL
//! ```
//! Run with:
//! `cargo run --example max30102-display-f3 --target thumbv7em-none-eabihf`

#![deny(unsafe_code)]
#![no_std]
#![no_main]

// panic handler
extern crate embedded_graphics;
extern crate panic_semihosting;

use cortex_m_rt::entry;
use embedded_graphics::fonts::Font6x8;
use embedded_graphics::prelude::*;
use f3::{
    hal::{delay::Delay, i2c::I2c, prelude::*, stm32f30x},
    led::Led,
};
use max3010x::{Led as MaxLed, Max3010x, TimeSlot};
use ssd1306::prelude::*;
use ssd1306::Builder;

use core::fmt::Write;
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
    let scl = gpiob.pb6.into_af4(&mut gpiob.moder, &mut gpiob.afrl);
    let sda = gpiob.pb7.into_af4(&mut gpiob.moder, &mut gpiob.afrl);

    let i2c = I2c::i2c1(dp.I2C1, (scl, sda), 100.khz(), clocks, &mut rcc.apb1);

    let manager = shared_bus::BusManager::<cortex_m::interrupt::Mutex<_>, _>::new(i2c);
    let mut disp: GraphicsMode<_> = Builder::new().connect_i2c(manager.acquire()).into();

    disp.init().unwrap();
    disp.flush().unwrap();

    let mut max30102 = Max3010x::new_max30102(manager.acquire());

    led.on();
    delay.delay_ms(50_u16);
    led.off();
    delay.delay_ms(50_u16);
    max30102.reset().unwrap();

    let mut max30102 = max30102.into_heart_rate().unwrap();
    max30102.set_pulse_amplitude(MaxLed::All, 15).unwrap();
    max30102.enable_fifo_rollover().unwrap();
    loop {
        // Blink LED 0 to check that everything is actually running.
        // If the LED 0 is off, something went wrong.
        led.on();
        delay.delay_ms(50_u16);
        led.off();
        delay.delay_ms(50_u16);

        let mut buffer: heapless::String<heapless::consts::U64> = heapless::String::new();

        let mut data = [0; 3];
        let read = max30102.read_fifo(&mut data).unwrap_or(0xFF);

        write!(buffer, "{}, {}, {}       ", data[0], data[1], data[2]).unwrap();
        disp.draw(
            Font6x8::render_str(&buffer)
                .with_stroke(Some(1u8.into()))
                .into_iter(),
        );

        disp.flush().unwrap();
    }
}
