# Additional example programs for several rust drivers

[![Build Status](https://travis-ci.org/eldruin/driver-examples.svg?branch=master)](https://travis-ci.org/eldruin/driver-examples)

This repository includes examples of using devices through these drivers:

| Device driver | Description                                               | Interface | Introductory blog post            |
|---------------|-----------------------------------------------------------|-----------|-----------------------------------|
|[Ad983x]       | Waveform generator / direct digital synthesizer (DDS).    | SPI       | [Intro blog post][blog-ad983x]    |
|[Ads1x1x]      | 12/16-bit Analog-to-digital (ADC) converters.             | I2C       | [Intro blog post][blog-ads1x1x]   |
|[Apds9960]     | Digital proximity, ambient light, RGB and gesture sensor. | I2C       |                                   |
|[Ds1307]       | Real-time clock (RTC) / calendar.                         | I2C       | [Intro blog post][blog-ds1307]    |
|[Ds323x]       | Extremely accurate real-time clock (RTC) / calendar.      | I2C / SPI |                                   |
|[Kxcj9]        | Tri-axis MEMS accelerometer.                              | I2C       | [Intro blog post][blog-kxcj9]     |
|[Eeprom24x]    | 24x series serial EEPROM devices.                         | I2C       | [Intro blog post][blog-eeprom24x] |
|[Lm75]         | Temperature sensor and thermal watchdog.                  | I2C       |                                   |
|[Max3010x]     | Pulse oximeter and heart-rate sensor.                     | I2C       |                                   |
|[Max44009]     | Ambient light sensor.                                     | I2C       |                                   |
|[Mcp4x]        | Digital potentiometers.                                   | SPI       |                                   |
|[Mcp49xx]      | 8/10/12-bit Digital-to-analog (DAC) converters.           | SPI       |                                   |
|[Mcp794xx]     | Real-time clock (RTC) / calendar.                         | I2C       | [Intro blog post][blog-mcp794xx]  |
|[Opt300x]      | Ambient light sensor.                                     | I2C       | [Intro blog post][blog-opt300x]   |
|[Pcf857x]      | 8/16-pin I/O port expanders.                              | I2C       |                                   |
|[Pwm-pca9685]  | 16-pin PWM port expander / LED driver.                    | I2C       |                                   |
|[Si4703]       | FM radio turners (receivers).                             | I2C       |                                   |
|[Tcs3472]      | RGBW light color sensor with IR filter.                   | I2C       |                                   |
|[Tmp006]       | Non-contact infrared (IR) thermopile temperature sensor.  | I2C       | [Intro blog post][blog-tmp006]    |
|[Tmp1x2]       | Temperature sensors.                                      | I2C       | [Intro blog post][blog-tmp1x2]    |
|[Veml6030]     | Ambient light sensor.                                     | I2C       |                                   |
|[Veml6040]     | RGBW light color sensor.                                  | I2C       |                                   |
|[Veml6070]     | Ultraviolet A (UVA) light sensor.                         | I2C       |                                   |
|[Veml6075]     | Ultraviolet A (UVA) and B (UVB) light sensor.             | I2C       | [Intro blog post][blog-veml6075]  |
|[W25]          | Winbond's W25 serial flash memory devices.                | SPI       |                                   |
|[Xca9548a]     | TCA9548A/PCA9548A I2C switches/multiplexers.              | I2C       |                                   |

These examples use either the STM32F3Discovery board or the STM32F103 "Blue pill" board.
At the beginning of each example the setup
and behavior is described. Many of them also use an SSD1306 OLED display.
You can get most of the modules used here on [AliExpress] generally for a very small price.

For example, to run the mcp41x-f3 example:
First, connect your discovery board per USB, then connect OpenOCD in a terminal with:
```
openocd -f interface/stlink-v2-1.cfg -f target/stm32f3x.cfg
```

Then on another terminal run:
```
git clone https://github.com/eldruin/driver-examples
cd driver-examples/stm32f3-discovery
cargo run --example mcp41x-f3
```

## License

Licensed under either of

 * Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or
   http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license ([LICENSE-MIT](LICENSE-MIT) or
   http://opensource.org/licenses/MIT)

at your option.

### Contributing

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall
be dual licensed as above, without any additional terms or conditions.

[Ad983x]: https://crates.io/crates/ad983x
[Ads1x1x]: https://crates.io/crates/ads1x1x
[Apds9960]: https://crates.io/crates/apds9960
[Ds1307]: https://crates.io/crates/ds1307
[Ds323x]: https://crates.io/crates/ds323x
[Kxcj9]: https://crates.io/crates/kxcj9
[Eeprom24x]: https://crates.io/crates/eeprom24x
[Lm75]: https://crates.io/crates/lm75
[Max3010x]: https://crates.io/crates/max3010x
[Max44009]: https://crates.io/crates/max44009
[Mcp4x]: https://crates.io/crates/mcp4x
[Mcp49xx]: https://crates.io/crates/mcp49xx
[Mcp794xx]: https://crates.io/crates/mcp794xx
[Opt300x]: https://crates.io/crates/Opt300x
[Pcf857x]: https://crates.io/crates/pcf857x
[Pwm-pca9685]: https://crates.io/crates/pwm-pca9685
[Si4703]: https://github.com/eldruin/si4703-rs
[Tcs3472]: https://crates.io/crates/tcs3472
[Tmp006]: https://crates.io/crates/tmp006
[Tmp1x2]: https://crates.io/crates/tmp1x2
[Veml6030]: https://crates.io/crates/veml6030
[Veml6040]: https://crates.io/crates/veml6040
[Veml6070]: https://crates.io/crates/veml6070
[Veml6075]: https://crates.io/crates/veml6075
[W25]: https://github.com/eldruin/w25-rs
[Xca9548a]: https://crates.io/crates/xca9548a

[blog-ad983x]: https://blog.eldruin.com/ad983x-waveform-generator-dds-driver-in-rust/
[blog-ads1x1x]: https://blog.eldruin.com/ads1x1x-analog-to-digital-converter-driver-in-rust/
[blog-ds1307]: https://blog.eldruin.com/ds1307-real-time-clock-rtc-driver-in-rust/
[blog-eeprom24x]: https://blog.eldruin.com/24x-serial-eeprom-driver-in-rust/
[blog-kxcj9]: https://blog.eldruin.com/kxcj9-kxcjb-tri-axis-mems-accelerator-driver-in-rust/
[blog-mcp794xx]: https://blog.eldruin.com/mcp794xx-real-time-clock-rtc-driver-in-rust
[blog-opt300x]: https://blog.eldruin.com/opt300x-ambient-light-sensor-driver-in-rust/
[blog-tmp006]: https://blog.eldruin.com/tmp006-contact-less-infrared-ir-thermopile-driver-in-rust/
[blog-tmp1x2]: https://blog.eldruin.com/tmp1x2-temperature-sensor-driver-in-rust/
[blog-veml6075]: https://blog.eldruin.com/veml6075-uva-uvb-uv-index-light-sensor-driver-in-rust/

[AliExpress]: https://www.aliexpress.com