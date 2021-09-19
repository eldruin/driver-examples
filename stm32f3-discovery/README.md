# Additional example programs for several rust drivers running on STM32F3 discovery board

These examples use the STM32F3Discovery board. At the beginning of each example the setup
and behavior is described. Many of them also use an SSD1306 OLED display.
You can get the modules used here on [AliExpress] generally for a very small price.

For example, to run the mcp41x-f3 example:
First, connect your discovery board per USB, then connect OpenOCD in a terminal with:

If you have not done this already, to use [cargo-embed][probe-rs] you need to update the firmware in your ST-Link with the [Stsw-link007][stlink-update] tool.
Then install `cargo-embed` with:
```
cargo install cargo-embed
```

Then on another terminal run:
```
git clone https://github.com/eldruin/driver-examples
cd driver-examples/stm32f3-discovery
cargo embed --example mcp41x-f3
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
[probe-rs]: https://probe.rs
[stlink-update]: https://www.st.com/en/development-tools/stsw-link007.html
