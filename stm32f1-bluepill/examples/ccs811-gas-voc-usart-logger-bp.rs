//! Continuously measure the eCO2 and eTVOC in the air, logs the values and sends
//! them through the serial interface every 10 seconds.
//! In order to compensate for the ambient temperature and humidity, an HDC2080
//! sensor is used.
//!
//! Introductory blog post with some pictures here:
//! https://blog.eldruin.com/ccs811-indoor-air-quality-sensor-driver-in-rust/
//!
//! This example is runs on the STM32F103 "Bluepill" board using I2C1 and USART1.
//!
//! To setup the serial communication, have a look at the discovery book:
//! https://rust-embedded.github.io/discovery/10-serial-communication/index.html
//!
//! ```
//! BP   <-> CCS811 <-> HDC2080 <-> Serial module
//! GND  <-> GND    <-> GND     <-> GND
//! 3.3V <-> VCC    <-> VCC     <-> VDD
//! PB8  <-> SCL    <-> SCL      
//! PB9  <-> SDA    <-> SDA      
//! PB6             <-> RX
//! GND  <-> nWAKE
//! 3.3V <-> RST
//! ```
//!
//! Run with:
//! `cargo run --example ccs811-gas-voc-usart-logger-bp`,

#![deny(unsafe_code)]
#![no_std]
#![no_main]

use core::fmt::Write;
use embedded_ccs811::{
    mode as Ccs811Mode, prelude::*, AlgorithmResult, Ccs811Awake, MeasurementMode,
    SlaveAddr as Ccs811SlaveAddr,
};
use embedded_hal::digital::v2::OutputPin;
use hdc20xx::{mode as Hdc20xxMode, Hdc20xx, SlaveAddr as Hdc20xxSlaveAddr};
use nb::block;
use panic_semihosting as _;
use rtic::app;
use rtic::cyccnt::U32Ext;
use shared_bus_rtic::SharedBus;
use stm32f1xx_hal::{
    delay::Delay,
    gpio::{
        gpiob::{PB8, PB9},
        gpioc::PC13,
        Alternate, OpenDrain, Output, PushPull, State,
    },
    i2c::{BlockingI2c, Mode},
    pac,
    prelude::*,
    serial,
};

const PERIOD: u32 = 1_000_000_000; // 10 seconds
type I2cBus = BlockingI2c<pac::I2C1, (PB8<Alternate<OpenDrain>>, PB9<Alternate<OpenDrain>>)>;

/*
 * shared-bus-rtic aggregate: multiple peripherals on a single i2c bus
 *
 * According to shared-bus-rtic docs:
 * Note that all of the drivers that use the same underlying bus **must** be stored within a single
 * resource (e.g. as one larger `struct`) within the RTIC resources. This ensures that RTIC will
 * prevent one driver from interrupting another while they are using the same underlying bus.
 */

pub struct I2cDevs {
    ccs811: Ccs811Awake<SharedBus<I2cBus>, Ccs811Mode::App>,
    hdc2080: Hdc20xx<SharedBus<I2cBus>, Hdc20xxMode::OneShot>,
}

#[app(device = stm32f1xx_hal::pac, peripherals = true, monotonic = rtic::cyccnt::CYCCNT)]
const APP: () = {
    struct Resources {
        led: PC13<Output<PushPull>>,
        i2c: I2cDevs,
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
        let cp = cortex_m::Peripherals::take().unwrap();
        let mut delay = Delay::new(cp.SYST, clocks);

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

        let mut gpioc = device.GPIOC.split(&mut rcc.apb2);
        let mut led = gpioc
            .pc13
            .into_push_pull_output_with_state(&mut gpioc.crh, State::Low);
        led.set_low().unwrap();

        let manager = shared_bus_rtic::new!(i2c, I2cBus);
        let mut hdc2080 = Hdc20xx::new(manager.acquire(), Hdc20xxSlaveAddr::default());
        let mut ccs811 = Ccs811Awake::new(manager.acquire(), Ccs811SlaveAddr::default());
        ccs811.software_reset().unwrap();
        delay.delay_ms(10_u16);

        let mut ccs811 = ccs811.start_application().ok().unwrap();
        let env = block!(hdc2080.read()).unwrap();
        ccs811
            .set_environment(env.temperature, env.humidity.unwrap_or(0.0))
            .unwrap();
        ccs811.set_mode(MeasurementMode::ConstantPower1s).unwrap();

        cx.schedule.measure(cx.start + PERIOD.cycles()).unwrap();

        let (mut tx, _rx) = serial.split();
        writeln!(tx, "start\r",).unwrap();
        init::LateResources {
            led,
            i2c: I2cDevs { ccs811, hdc2080 },
            tx,
        }
    }

    #[task(resources = [led, i2c, tx], schedule = [measure])]
    fn measure(cx: measure::Context) {
        // Use the safe local `static mut` of RTIC
        static mut LED_STATE: bool = false;
        static mut ENV: [(f32, f32); 1200] = [(0.0, 0.0); 1200];
        static mut MEASUREMENTS: [AlgorithmResult; 1200] = [AlgorithmResult {
            eco2: 0,
            etvoc: 0,
            raw_current: 0,
            raw_voltage: 0,
        }; 1200];
        static mut INDEX: usize = 0;

        if *LED_STATE {
            cx.resources.led.set_high().unwrap();
            *LED_STATE = false;
        } else {
            cx.resources.led.set_low().unwrap();
            *LED_STATE = true;
        }

        let default = AlgorithmResult::default();
        if *INDEX < MEASUREMENTS.len() {
            let data = block!(cx.resources.i2c.ccs811.data()).unwrap_or(default);
            MEASUREMENTS[*INDEX] = data;
            let env = block!(cx.resources.i2c.hdc2080.read()).unwrap();
            let temp = env.temperature;
            let humidity = env.humidity.unwrap_or(0.0);
            ENV[*INDEX] = (temp, humidity);
            *INDEX += 1;
            cx.resources
                .i2c
                .ccs811
                .set_environment(temp, humidity)
                .unwrap();
        }
        writeln!(cx.resources.tx, "\rstart\r",).unwrap();
        for i in 0..*INDEX {
            let data = MEASUREMENTS[i];
            let env = if i == 0 { (0.0, 0.0) } else { ENV[i - 1] };
            writeln!(
                cx.resources.tx,
                "{},{},{},{},{},{:.2},{:.2}\r",
                i, data.eco2, data.etvoc, data.raw_current, data.raw_voltage, env.0, env.1
            )
            .unwrap();
        }
        cx.schedule.measure(cx.scheduled + PERIOD.cycles()).unwrap();
    }

    extern "C" {
        fn EXTI0();
    }
};
