# Additional example programs for several rust drivers

[![Build Status](https://travis-ci.org/eldruin/driver-examples.svg?branch=master)](https://travis-ci.org/eldruin/driver-examples)

This repository includes examples of using devices through these drivers:

| Device driver | Description                                               | Interface |
|---------------|-----------------------------------------------------------|-----------|
|[Ads1x1x]      | 12/16-bit Analog-to-digital (ADC) converters.             | I2C       |
|[Apds9960]     | Digital proximity, ambient light, RGB and gesture sensor. | I2C       |
|[Lm75]         | Temperature sensor and thermal watchdog.                  | I2C       |
|[Mcp4x]        | Digital potentiometers.                                   | SPI       |
|[Mcp49x]       | 8/10/12-bit Digital-to-analog (DAC) converters.           | SPI       |
|[Pcf857x]      | 8/16-pin I/O port expanders.                              | I2C       |
|[Pwm-pca9685]  | 16-pin PWM port expander / LED driver.                    | I2C       |
|[Tmp006]       | Non-contact infrared (IR) thermopile temperature sensor   | I2C       |

[Ads1x1x]: https://crates.io/crates/ads1x1x
[Apds9960]: https://crates.io/crates/apds9960
[Lm75]: https://crates.io/crates/lm75
[Mcp4x]: https://crates.io/crates/mcp4x
[Mcp49x]: https://github.com/eldruin/mcp49x-rs
[Pcf857x]: https://crates.io/crates/pcf857x
[Pwm-pca9685]: https://crates.io/crates/pwm-pca9685
[Tmp006]: https://crates.io/crates/tmp006

These examples use the STM32F3Discovery board. At the beginning of each example the setup
and behavior is described.

For example, to run the f3-mcp41x example:
First, connect your discovery board per USB, then connect OpenOCD in a terminal with:
```
openocd -f interface/stlink-v2-1.cfg -f target/stm32f3x.cfg
```

Then on another terminal run:
```
git clone https://github.com/eldruin/driver-examples
cd driver-examples
cargo run --example f3-mcp41x
```

## License

Licensed under either of

 * Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or
   http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license ([LICENSE-MIT](LICENSE-MIT) or
   http://opensource.org/licenses/MIT) at your option.

### Contributing

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall
be dual licensed as above, without any additional terms or conditions.
