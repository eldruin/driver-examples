//! This plays the final part of Beethoven's ninth symphony given by
//! its MIDI tones using an AD9833 waveform generator / direct digital synthesizer.
//!
//! You can see a video of this running here:
//! https://blog.eldruin.com/ad983x-waveform-generator-dds-driver-in-rust/
//!
//! This example is runs on the STM32F3 Discovery board using SPI1.
//!
//! ```
//! F3   <-> AD9833  <-> LM386
//! GND  <-> VSS     <-> GND
//! 3.3V <-> VDD
//! 5V               <-> VCC
//! PA5  <-> CLK
//! PA7  <-> DAT
//! PB5  <-> FSYNC
//!          OUT     <-> IN
//! ```
//!
//! You will need an amplifier like the LM386 or similar and a speaker.
//!
//! Run with:
//! `cargo run --example ad9833-midi-player-f3 --target thumbv7em-none-eabihf`,

#![deny(unsafe_code)]
#![no_std]
#![no_main]

// panic handler
extern crate panic_semihosting;

use ad983x::{Ad983x, FrequencyRegister, MODE};
use cortex_m_rt::entry;
use f3::{
    hal::{delay::Delay, prelude::*, spi::Spi, stm32f30x},
    led::Led,
};
use libm;

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

    chip_select.set_high();

    let mut synth = Ad983x::new_ad9833(spi, chip_select);
    synth.reset().unwrap();
    synth.enable().unwrap();

    let mut current_register = FrequencyRegister::F0;
    let mut table = MidiTable::default();
    loop {
        // Blink LED 0 to check that everything is actually running.
        // If the LED 0 does not blink, something went wrong.
        led.on();
        delay.delay_ms(50_u16);
        led.off();
        delay.delay_ms(50_u16);

        let midi_number = table.next().unwrap_or(0);
        let midi_number = f64::from(midi_number);
        let frequency_hz = libm::pow(2.0, (midi_number - 69.0) / 12.0) * 440.0;
        let mclk_hz = 25_000_000.0;
        let synth_value = frequency_hz * f64::from(1 << 28) / mclk_hz;

        // To ensure a smooth transition, set the frequency in the frequency
        // register that is not currently in use, then switch to it.
        let opposite = get_opposite(current_register);
        synth.set_frequency(opposite, synth_value as u32).unwrap();
        synth.select_frequency(opposite).unwrap();
        current_register = opposite;
    }
}

fn get_opposite(register: FrequencyRegister) -> FrequencyRegister {
    match register {
        FrequencyRegister::F0 => FrequencyRegister::F1,
        FrequencyRegister::F1 => FrequencyRegister::F0,
    }
}

#[derive(Debug, Default)]
struct MidiTable {
    position: usize,
    duration_counter: usize,
}

impl Iterator for MidiTable {
    type Item = u32;

    fn next(&mut self) -> Option<Self::Item> {
        let mut silence = None;
        let note_duration = Self::NOTES[self.position].1;
        let should_start_over = self.position == (Self::NOTES.len() - 1);
        if should_start_over {
            self.position = 0;
            self.duration_counter = 0;
        } else if self.duration_counter == note_duration {
            // insert silence after every note
            self.duration_counter += 1;
            silence = Some(0);
        } else if self.duration_counter > note_duration {
            self.position += 1;
            self.duration_counter = 0;
        } else {
            self.duration_counter += 1;
        }
        let tone = Some(Self::NOTES[self.position].0);
        silence.or(tone)
    }
}

impl MidiTable {
    const NOTES: [(u32, usize); 69] = [
        (76, 2),
        (76, 2),
        (77, 2),
        (79, 2),
        //
        (79, 2),
        (77, 2),
        (76, 2),
        (74, 2),
        //
        (72, 2),
        (72, 2),
        (74, 2),
        (76, 2),
        //
        (76, 2),
        (0, 1),
        (74, 1),
        (74, 3),
        (0, 1),
        //
        (76, 2),
        (76, 2),
        (77, 2),
        (79, 2),
        //
        (79, 2),
        (77, 2),
        (76, 2),
        (74, 2),
        //
        (72, 2),
        (72, 2),
        (74, 2),
        (76, 2),
        //
        (74, 2),
        (0, 1),
        (72, 1),
        (72, 3),
        (0, 1),
        //
        (74, 2),
        (74, 2),
        (76, 2),
        (72, 2),
        //
        (74, 2),
        (76, 1),
        (77, 1),
        (76, 2),
        (72, 2),
        //
        (74, 2),
        (76, 1),
        (77, 1),
        (76, 2),
        (74, 2),
        //
        (72, 2),
        (74, 2),
        (67, 2),
        (76, 4),
        //
        (0, 1),
        (76, 2),
        (77, 2),
        (79, 2),
        //
        (79, 2),
        (77, 2),
        (76, 2),
        (74, 2),
        //
        (72, 2),
        (72, 2),
        (74, 2),
        (76, 2),
        //
        (74, 2),
        (0, 1),
        (72, 1),
        (72, 3),
        //
        (0, 10),
    ];
}
