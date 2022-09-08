# Wifi and Bluetooth LE on ESP32-C3 and ESP32 (on bare-metal Rust)

## About

This is experimental and work-in-progress! You are welcome to experiment with it and contribute but probably shouldn't use this for something real yet.

WiFi / BTLE coexistence is implemented but currently only works (to some extend) on ESP32-C3.

THIS CURRENTLY DOESN'T WORK WITH THE XTENSA ENABLED RUST COMPILER 1.63.0.2

This uses the WiFi driver found in https://github.com/espressif/esp-wireless-drivers-3rdparty

## Version used

esp-wireless-drivers-3rdparty-055f1ef49d0cb72c24bd492fbbdd37497a90bdae
45701c0

https://github.com/espressif/esp-wireless-drivers-3rdparty/archive/45701c0.zip

## Examples

- dhcp
  - set SSID and PASSWORD env variable
  - gets an ip address via DHCP
  - performs an HTTP get request to some "random" server

- ble
    - starts Bluetooth advertising
    - offers two services (one is read/write, one is write only)
    - this uses a toy level BLE stack - might not work with every BLE central device (tested with Android and Windows Bluetooth LE Explorer)

| Command                                                                                                                      | Chip    |
| ---------------------------------------------------------------------------------------------------------------------------- | ------- |
| `cargo "+nightly" run --example ble_esp32c3 --release --target riscv32imc-unknown-none-elf --features "esp32c3,ble"`  | ESP32-C3 |
| `cargo "+nightly" run --example dhcp_esp32c3 --release --target riscv32imc-unknown-none-elf --features "esp32c3,embedded-svc,wifi"` | ESP32-C3 |
| `cargo "+nightly" run --example coex_esp32c3 --release --target riscv32imc-unknown-none-elf --features "esp32c3,embedded-svc,wifi,ble"` | ESP32-C3 |
| `cargo "+esp" run --example ble_esp32 --release --target xtensa-esp32-none-elf --features "esp32,ble"`              | ESP32   |
| `cargo "+esp" run --example dhcp_esp32 --release --target xtensa-esp32-none-elf --features "esp32,embedded-svc,wifi"`             | ESP32   |

Additional you can specify these features
|Feature|Meaning|
|---|---|
|wifi_logs|logs the WiFi logs from the driver at log level info|
|dump_packets|dumps some packet info at log level info|
|utils|Provide utilities for smoltcp initialization, this is a default feature|
|embedded-svc|Provides a (very limited) implementation of the `embedded-svc` WiFi trait, includes `utils` feature|
|ble|Enable BLE support|
|wifi|Enable WiFi support|

In general you should use the release profile since otherwise the performance is quite bad.

## What works?

- scanning for WiFi access points
- connect to WiFi access point
- providing an HCI interface

## Notes on ESP32-C3 support

- uses SYSTIMER as the main timer
- doesn't work in direct-boot mode

## Notes on ESP32 support

This is even more experimental than support for ESP32-C3.

- The WiFi logs only print the format string - not the actual values.
- Also there might be some packet loss and a bit worse performance than on ESP32-C3 currently.
- The code runs on a single core and might currently not be multi-core safe!

On ESP32 currently TIMG1/TIMER0 is used as the main timer so you can't use it for anything else.
Additionally it uses CCOMPARE0 - so don't touch that, too.

## Directory Structure

- `src/timer-espXXX.rs`: systimer code used for timing and task switching
- `src/preemt/`: a bare minimum RISCV and Xtensa round-robin task scheduler
- `src/log/`: code used for logging
- `src/binary/`: generated bindings to the WiFi driver (per chip)
- `src/compat/`: code needed to emulate enough of an (RT)OS to use the driver
  - `common.rs`: basics like semaphores and recursive mutexes
  - `timer_compat.rs`: code to emulate timer related functionality
- `headers`: headers found in the WiFi driver archive (bindings are generated from these)
- `libs/espXXX`: static libraries found in the WiFi driver archive (these get linked into the binary)
- `mkbindings.bat`: generate the bindings / just calls `bindgen`
- `ld/espXXX/rom_functions.x`: the WiFi driver uses some of these so it needs to get linked
- `ld/esp32/wifi-link.x`: the main linker script for ESP32 - needs to get cleaned up and ideally the changes move to ESP-HAL
- `examples/dhcp_espXXX/main.rs`: example using the code (per chip)

## Missing / To be done

- lots of refactoring
- make CoEx work on ESP32
- esp-now
- powersafe support
- maybe SoftAP

## License

Licensed under either of:

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in
the work by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without
any additional terms or conditions.
