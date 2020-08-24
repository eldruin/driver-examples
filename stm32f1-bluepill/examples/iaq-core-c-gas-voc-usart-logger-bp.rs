//! Measures the CO2 and TVOC equivalents in the air with an iAQ-Core-C module,
//! logs the values and sends them through the serial interface every 10 seconds.
//!
//! This example is runs on the STM32F103 "Bluepill" board using I2C1 and USART1.
//!
//! To setup the serial communication, have a look at the discovery book:
//! https://rust-embedded.github.io/discovery/10-serial-communication/index.html
//!
//! ```
//! BP   <-> iAQ-Core-C <-> Serial module
//! GND  <-> GND        <-> GND
//! 3.3V <-> VCC        <-> VDD
//! PB8  <-> SCL     
//! PB9  <-> SDA     
//! PB6                 <-> RX
//! ```
//!
//! Run with:
//! `cargo run --example iaq-core-c-gas-voc-usart-logger-bp`,

#![deny(unsafe_code)]
#![no_std]
#![no_main]

use core::fmt::Write;
use nb::block;
use panic_semihosting as _;
use stm32f1xx_hal::{
    gpio::{
        gpiob::{PB8, PB9},
        gpioc::PC13,
        Alternate, OpenDrain, Output, PushPull, State,
    },
    i2c::{BlockingI2c, DutyCycle, Mode},
    pac,
    prelude::*,
    serial,
};

use iaq_core::{IaqCore, Measurement};

use embedded_hal::digital::v2::OutputPin;
use panic_semihosting as _;
use rtic::app;
use rtic::cyccnt::U32Ext;

const PERIOD: u32 = 1_000_000_000; // 10 seconds
type I2cBus = BlockingI2c<pac::I2C1, (PB8<Alternate<OpenDrain>>, PB9<Alternate<OpenDrain>>)>;

#[app(device = stm32f1xx_hal::pac, peripherals = true, monotonic = rtic::cyccnt::CYCCNT)]
const APP: () = {
    struct Resources {
        led: PC13<Output<PushPull>>,
        sensor: IaqCore<I2cBus>,
        tx: serial::Tx<pac::USART1>,
    }

    #[init(schedule = [measure])]
    fn init(cx: init::Context) -> init::LateResources {
        let mut core = cx.core;
        core.DWT.enable_cycle_counter();

        let device: stm32f1xx_hal::stm32::Peripherals = cx.device;

        let mut flash = device.FLASH.constrain();
        let mut rcc = device.RCC.constrain();
        let mut afio = device.AFIO.constrain(&mut rcc.apb2);
        let clocks = rcc
            .cfgr
            .use_hse(8.mhz())
            .sysclk(72.mhz())
            .pclk1(36.mhz())
            .freeze(&mut flash.acr);
        let _cp = cortex_m::Peripherals::take().unwrap();

        let mut gpiob = device.GPIOB.split(&mut rcc.apb2);

        let scl = gpiob.pb8.into_alternate_open_drain(&mut gpiob.crh);
        let sda = gpiob.pb9.into_alternate_open_drain(&mut gpiob.crh);

        let i2c = BlockingI2c::i2c1(
            device.I2C1,
            (scl, sda),
            &mut afio.mapr,
            Mode::Standard {
                frequency: 100_000.hz(),
            },
            clocks,
            &mut rcc.apb1,
            1000,
            10,
            1000,
            1000,
        );
        let tx = gpiob.pb6.into_alternate_push_pull(&mut gpiob.crl);
        let rx = gpiob.pb7;
        let serial = serial::Serial::usart1(
            device.USART1,
            (tx, rx),
            &mut afio.mapr,
            serial::Config::default().baudrate(115200.bps()),
            clocks,
            &mut rcc.apb2,
        );
        let sensor = IaqCore::new(i2c);

        let mut gpioc = device.GPIOC.split(&mut rcc.apb2);
        let mut led = gpioc
            .pc13
            .into_push_pull_output_with_state(&mut gpioc.crh, State::Low);
        led.set_low().unwrap();

        cx.schedule.measure(cx.start + PERIOD.cycles()).unwrap();

        let (mut tx, _rx) = serial.split();
        writeln!(tx, "start\r",).unwrap();
        init::LateResources { led, sensor, tx }
    }

    #[task(resources = [led, sensor, tx], schedule = [measure])]
    fn measure(cx: measure::Context) {
        // Use the safe local `static mut` of RTIC
        static mut LED_STATE: bool = false;
        static mut MEASUREMENTS: [Measurement; 2400] = [Measurement {
            co2: 0,
            tvoc: 0,
            resistance: 0,
        }; 2400];
        static mut INDEX: usize = 0;

        if *LED_STATE {
            cx.resources.led.set_high().unwrap();
            *LED_STATE = false;
        } else {
            cx.resources.led.set_low().unwrap();
            *LED_STATE = true;
        }

        let default = Measurement {
            co2: 1,
            tvoc: 1,
            resistance: 1,
        };
        if *INDEX < MEASUREMENTS.len() {
            let data = block!(cx.resources.sensor.data()).unwrap_or(default);
            MEASUREMENTS[*INDEX] = data;
            *INDEX += 1;
        }
        for i in 0..*INDEX {
            let data = MEASUREMENTS[i];
            writeln!(
                cx.resources.tx,
                "{},{},{},{}\r",
                i, data.co2, data.tvoc, data.resistance
            )
            .unwrap();
        }
        cx.schedule.measure(cx.scheduled + PERIOD.cycles()).unwrap();
    }

    extern "C" {
        fn EXTI0();
    }
};
