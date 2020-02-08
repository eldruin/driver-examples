//! Continuously read the accelerometer and gyroscope and print
//! the data to an SSD1306 OLED display.
//!
//! This example is runs on the STM32F103 "Bluepill" board using I2C1.
//!
//! ```
//! BP   <-> BMI160 <-> Display
//! GND  <-> GND    <-> GND
//! 3.3V <-> VCC    <-> VDD
//! PB8  <-> SCL    <-> SCL
//! PB9  <-> SDA    <-> SDA
//! ```
//!
//! Run with:
//! `cargo run --example bmi160-imu-display-bp`,

#![deny(unsafe_code)]
#![no_std]
#![no_main]

use core::fmt::Write;
use cortex_m_rt::entry;
use embedded_graphics::{fonts::Font6x8, prelude::*};
use embedded_hal::digital::v2::OutputPin;
use panic_semihosting as _;
use ssd1306::{prelude::*, Builder};
use stm32f1xx_hal::{
    delay::Delay,
    i2c::{BlockingI2c, DutyCycle, Mode},
    pac,
    prelude::*,
};

use bmi160::{
    AccelerometerPowerMode, Bmi160, Data, GyroscopePowerMode, Sensor3DData, SensorSelector,
    SlaveAddr,
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
    let sda = gpiob.pb9.into_alternate_open_drain(&mut gpiob.crh);

    let i2c = BlockingI2c::i2c1(
        dp.I2C1,
        (scl, sda),
        &mut afio.mapr,
        Mode::Fast {
            frequency: 100_000,
            duty_cycle: DutyCycle::Ratio2to1,
        },
        clocks,
        &mut rcc.apb1,
        1000,
        10,
        1000,
        1000,
    );

    let mut gpioc = dp.GPIOC.split(&mut rcc.apb2);
    let mut led = gpioc.pc13.into_push_pull_output(&mut gpioc.crh);
    let mut delay = Delay::new(cp.SYST, clocks);

    let manager = shared_bus::BusManager::<cortex_m::interrupt::Mutex<_>, _>::new(i2c);
    let mut disp: GraphicsMode<_> = Builder::new().connect_i2c(manager.acquire()).into();

    disp.init().unwrap();
    disp.flush().unwrap();

    let mut imu = Bmi160::new_with_i2c(manager.acquire(), SlaveAddr::Alternative(true));
    imu.set_accel_power_mode(AccelerometerPowerMode::Normal)
        .unwrap();
    imu.set_gyro_power_mode(GyroscopePowerMode::Normal).unwrap();

    let mut lines: [heapless::String<heapless::consts::U32>; 2] =
        [heapless::String::new(), heapless::String::new()];
    let default_3ddata = Sensor3DData {
        x: -1,
        y: -1,
        z: -1,
    };
    let default_data = Data {
        accel: Some(default_3ddata),
        gyro: Some(default_3ddata),
        magnet: None,
        time: None,
    };

    loop {
        // Blink LED 0 to check that everything is actually running.
        // If the LED 0 is off, something went wrong.
        led.set_high().unwrap();
        delay.delay_ms(50_u16);
        led.set_low().unwrap();
        delay.delay_ms(50_u16);

        let data = imu
            .data(SensorSelector::new().accel().gyro())
            .unwrap_or(default_data);
        let accel = data.accel.unwrap();
        let gyro = data.gyro.unwrap();

        lines[0].clear();
        lines[1].clear();
        write!(lines[0], "acc: x {} y {} z {}", accel.x, accel.y, accel.z).unwrap();
        write!(lines[1], "gyr: x {} y {} z {}", gyro.x, gyro.y, gyro.z).unwrap();
        disp.draw(
            Font6x8::render_str(&lines[0])
                .with_stroke(Some(1u8.into()))
                .into_iter(),
        );
        disp.draw(
            Font6x8::render_str(&lines[1])
                .with_stroke(Some(1u8.into()))
                .translate(Coord::new(0, 12))
                .into_iter(),
        );
        disp.flush().unwrap();
    }
}
