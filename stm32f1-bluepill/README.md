# Additional example programs for several rust drivers running on STM32F103 "bluepill" board

These examples use the STM32F103 "Bluepill" board. At the beginning of each example the setup
and behavior is described. Many of them also use an SSD1306 OLED display.
You can buy this board and most of the modules used here on [AliExpress] generally for a very small price.

For example, to run the veml6070 example:
First, connect your discovery board per USB, then connect OpenOCD in a terminal with:
```
openocd -f interface/stlink-v2.cfg -f target/stm32f1x.cfg
```

Then on another terminal run:
```
git clone https://github.com/eldruin/driver-examples
cd driver-examples/stm32f1-bluepill
cargo run --example veml6070-uv-display-bp
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

[AliExpress]: https://www.aliexpress.com