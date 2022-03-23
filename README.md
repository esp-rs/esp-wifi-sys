# Wifi on ESP32C3 (on bare-metal Rust)

## About

This is experimental and work-in-progress! You are welcome to contribute but probably shouldn't use this for something real yet.

This uses the WiFi driver found in https://github.com/espressif/esp-wireless-drivers-3rdparty

## Version used

esp-wireless-drivers-3rdparty-055f1ef49d0cb72c24bd492fbbdd37497a90bdae
45701c0

https://github.com/espressif/esp-wireless-drivers-3rdparty/archive/45701c0.zip

## Example

- dhcp
    - set SSID and PASSWORD env variable
    - gets an ip address via DHCP
    - it prints the ip address it gets
    - if everything works you should be able to ping and connect to port 4321

## What works?

- scanning for WiFi access points
- connect to WiFi access point

## Directory Structure

- src/timer.rs: systimer code used for timing and task switching
- src/preemt/: a bare minimum RISCV round-robin task scheduler
- src/log/: code used for logging
- src/binary/: generated bindings to the WiFi driver
- src/compat/: code needed to emulate enough of an (RT)OS to use the driver
    - malloc.rs: a homegrown allocator - this is NOT used on the Rust side (the Rust side of this is currently no-alloc)
    - common.rs: basics like semaphores and recursive mutexes
    - timer_compat.rs: code to emulate timer related functionality
- headers: headers found in the WiFi driver archive (bindings are generated from these)
- libs: static libraries found in the WiFi driver archive (these get linked into the binary)
- mkbindings.bat: generate the bindings / just calls `bindgen`
- rom_functions.x: the WiFi driver uses some of these so it needs to get linked
- esp32c3-wifi-link.x: the main linker script - needs to get cleaned up
- examples/dhcp.rs: example using the code

## Missing / To be done
- separating chip specific / architecture specific code (and create implementations for others)
- Bluetooth (and coex)
- esp-now
- powersafe support

## License

Licensed under either of:

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in
the work by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without
any additional terms or conditions.
