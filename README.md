# Additional example programs for the [mcp4x] crate

[![Build Status](https://travis-ci.org/eldruin/mcp4x-examples.svg?branch=master)](https://travis-ci.org/eldruin/mcp4x-examples)

[mcp4x]: https://crates.io/crates/mcp4x

This repository contains additional example programs using the MCP4X SPI
digital potentiometers with an STM32F3Discovery board.

For example, to run the f3-mcp41x example:
First, connect your discovery board per USB, then connect OpenOCD in a terminal with:
```
openocd -f interface/stlink-v2-1.cfg -f target/stm32f3x.cfg
```

Then on another terminal run:
```
git clone https://github.com/eldruin/mcp4x-examples
cd mcp4x-examples
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

