//! Seek an FM radio channel when pressing two buttons "Seek down" / "Seek up"
//! using an Si4703 FM radio receiver (turner).
//!
//! Introductory blog post with some pictures here:
//! https://blog.eldruin.com/si4703-fm-radio-receiver-driver-in-rust/
//!
//! This example is runs on the STM32F103 "Bluepill" board using I2C1.
//!
//! ```
//! BP    <-> Si4703
//! GND   <-> GND
//! +3.3V <-> VCC
//! PB8   <-> SCLK
//! PB9   <-> SDIO
//! PB7   <-> RST
//! PB6   <-> GPIO2
//! PB10             <-> Seek up button   <-> +3.3V
//! PB11             <-> Seek down button <-> +3.3V
//! ```
//!
//! Run with:
//! `cargo embed --example si4703-fm-radio-bp`,

#![deny(unsafe_code)]
#![no_std]
#![no_main]

use cortex_m_rt::entry;
use embedded_hal::digital::v2::{InputPin, OutputPin};
use nb::block;
use panic_rtt_target as _;
use rtt_target::{rprintln, rtt_init_print};
use si4703::{
    reset_and_select_i2c_method1 as reset_si4703, ChannelSpacing, DeEmphasis, SeekDirection,
    SeekMode, Si4703, Volume,
};
use stm32f1xx_hal::{
    delay::Delay,
    i2c::{BlockingI2c, DutyCycle, Mode},
    pac,
    prelude::*,
};

#[entry]
fn main() -> ! {
    rtt_init_print!();
    rprintln!("Si4703 example");
    let cp = cortex_m::Peripherals::take().unwrap();
    let dp = pac::Peripherals::take().unwrap();

    let mut flash = dp.FLASH.constrain();
    let mut rcc = dp.RCC.constrain();

    let clocks = rcc.cfgr.freeze(&mut flash.acr);
    let mut afio = dp.AFIO.constrain(&mut rcc.apb2);
    let mut gpiob = dp.GPIOB.split(&mut rcc.apb2);

    let scl = gpiob.pb8.into_alternate_open_drain(&mut gpiob.crh);
    let mut sda = gpiob.pb9.into_push_pull_output(&mut gpiob.crh);
    let mut rst = gpiob.pb7.into_push_pull_output(&mut gpiob.crl);
    let stcint = gpiob.pb6.into_pull_up_input(&mut gpiob.crl);
    let seekdown = gpiob.pb11.into_pull_down_input(&mut gpiob.crh);
    let seekup = gpiob.pb10.into_pull_down_input(&mut gpiob.crh);
    let mut gpioc = dp.GPIOC.split(&mut rcc.apb2);
    let mut led = gpioc.pc13.into_push_pull_output(&mut gpioc.crh);
    let mut delay = Delay::new(cp.SYST, clocks);

    reset_si4703(&mut rst, &mut sda, &mut delay).unwrap();
    let sda = sda.into_alternate_open_drain(&mut gpiob.crh);
    let i2c = BlockingI2c::i2c1(
        dp.I2C1,
        (scl, sda),
        &mut afio.mapr,
        Mode::Fast {
            frequency: 400_000.hz(),
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

    let mut radio = Si4703::new(manager.acquire());
    radio.enable_oscillator().unwrap();
    delay.delay_ms(500_u16);
    radio.enable().unwrap();
    delay.delay_ms(110_u16);

    radio.set_volume(Volume::Dbfsm28).unwrap();
    radio.set_deemphasis(DeEmphasis::Us50).unwrap();
    radio.set_channel_spacing(ChannelSpacing::Khz100).unwrap();
    radio.unmute().unwrap();
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
            let direction = if should_seek_down {
                SeekDirection::Down
            } else {
                SeekDirection::Up
            };

            block!(radio.seek_with_stc_int_pin(SeekMode::Wrap, direction, &stcint)).unwrap();
        }
    }
}
