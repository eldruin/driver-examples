//! Seek an FM radio channel when pressing two buttons "Seek down" / "Seek up"
//! using an Si4703 FM radio receiver (turner) and display the channel
//! frequency in an SSD1306 OLED display
//!
//! This example is runs on the STM32F103 "Bluepill" board using I2C1.
//!
//! ```
//! BP    <-> Si4703 <-> Display
//! GND   <-> GND    <-> GND
//! +3.3V <-> VCC    <-> VCC
//! PB8   <-> SCLK   <-> SCL
//! PB9   <-> SDIO   <-> SDA
//! PB7   <-> RST
//! PB6   <-> GPIO2
//! PB10                        <-> Seek up button   <-> +3.3V
//! PB11                        <-> Seek down button <-> +3.3V
//! ```
//!
//! Run with:
//! `cargo run --example si4703-fm-radio-bp`,

#![deny(unsafe_code)]
#![no_std]
#![no_main]

use core::fmt::Write;
use cortex_m_rt::entry;
use embedded_graphics::{fonts::Font6x8, prelude::*};
use embedded_hal::digital::v2::{InputPin, OutputPin};
use panic_semihosting as _;
use si4703::{
    reset as reset_si4703, ChannelSpacing, DeEmphasis, ErrorWithPin, SeekDirection, SeekMode,
    Si4703, Volume,
};
use ssd1306::{prelude::*, Builder};
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
    let mut sda = gpiob.pb9.into_alternate_open_drain(&mut gpiob.crh);
    let mut rst = gpiob.pb7.into_push_pull_output(&mut gpiob.crl);
    let stcint = gpiob.pb6.into_pull_up_input(&mut gpiob.crl);
    let seekdown = gpiob.pb11.into_pull_down_input(&mut gpiob.crh);
    let seekup = gpiob.pb10.into_pull_down_input(&mut gpiob.crh);
    let mut gpioc = dp.GPIOC.split(&mut rcc.apb2);
    let mut led = gpioc.pc13.into_push_pull_output(&mut gpioc.crh);
    let mut delay = Delay::new(cp.SYST, clocks);

    reset_si4703(&mut rst, &mut sda, &mut delay).unwrap();
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
    let manager = shared_bus::BusManager::<cortex_m::interrupt::Mutex<_>, _>::new(i2c);
    let mut disp: GraphicsMode<_> = Builder::new().connect_i2c(manager.acquire()).into();

    disp.init().unwrap();
    disp.flush().unwrap();

    let mut radio = Si4703::new(manager.acquire());
    radio.enable_oscillator().unwrap();
    delay.delay_ms(500_u16);
    radio.enable().unwrap();
    delay.delay_ms(110_u16);

    radio.set_volume(Volume::Dbfsm28).unwrap();
    radio.set_deemphasis(DeEmphasis::Us50).unwrap();
    radio.set_channel_spacing(ChannelSpacing::Khz100).unwrap();
    radio.unmute().unwrap();
    let mut buffer: heapless::String<heapless::consts::U64> = heapless::String::new();
    loop {
        // Blink LED 0 every time a new seek is started
        // to check that everything is actually running.
        led.set_low().unwrap();
        delay.delay_ms(50_u16);
        led.set_high().unwrap();
        delay.delay_ms(50_u16);
        let should_seek_down = seekdown.is_high().unwrap();
        let should_seek_up = seekup.is_high().unwrap();
        if should_seek_down || should_seek_up {
            buffer.clear();
            write!(buffer, "Seeking...   ").unwrap();
            disp.draw(
                Font6x8::render_str(&buffer)
                    .with_stroke(Some(1u8.into()))
                    .into_iter(),
            );
            disp.flush().unwrap();
            let mode = SeekMode::Wrap;
            let direction = if should_seek_down {
                SeekDirection::Down
            } else {
                SeekDirection::Up
            };

            loop {
                buffer.clear();
                match radio.seek_with_stc_int_pin(mode, direction, &stcint) {
                    Err(nb::Error::WouldBlock) => {
                        let channel = radio.get_channel().unwrap_or(-1.0);
                        write!(buffer, "Trying {:1}    ", channel).unwrap();
                        disp.draw(
                            Font6x8::render_str(&buffer)
                                .with_stroke(Some(1u8.into()))
                                .into_iter(),
                        );
                        disp.flush().unwrap();
                    }
                    Err(nb::Error::Other(ErrorWithPin::SeekFailed)) => {
                        write!(buffer, "Seek Failed!  ").unwrap();
                        break;
                    }
                    Err(_) => {
                        write!(buffer, "Error!     ").unwrap();
                        break;
                    }
                    Ok(_) => {
                        let channel = radio.get_channel().unwrap_or(-1.0);
                        write!(buffer, "Found! {:1}    ", channel).unwrap();
                        break;
                    }
                }
            }

            disp.draw(
                Font6x8::render_str(&buffer)
                    .with_stroke(Some(1u8.into()))
                    .into_iter(),
            );
            disp.flush().unwrap();
        }
    }
}
