# esp-wifi

Wi-Fi/BTLE coexistence is implemented but currently only works (to some extent) on ESP32-C3 and ESP32-S3. In general COEX shouldn't be used currently.

Minimum supported Rust compiler version: 1.72.0.0

This uses the WiFi drivers from https://github.com/esp-rs/esp-wireless-drivers-3rdparty

## Version used

v5.1-rc2-4-gc570f67461 commit c570f674610479fc5e070c8db6d181b73ddf60a8

https://github.com/esp-rs/esp-wireless-drivers-3rdparty/ (commit 976e9cc6c0725e8325a7e3a362d113559238c45c)

## Current support

If a cell contains am em dash (&mdash;) this means that the particular feature is not present for a chip. A check mark (✓) means that some driver implementation exists. An empty cell means that the feature is present in the chip but not implemented yet.

|          | [Wifi](https://github.com/esp-rs/esp-wifi/issues/94) | [BLE](https://github.com/esp-rs/esp-wifi/issues/93) | [Coex](https://github.com/esp-rs/esp-wifi/issues/92) | ESP-NOW |
| :------: | :--------------------------------------------------: | :-------------------------------------------------: | :--------------------------------------------------: | :-----: |
|  ESP32   |                          ✓                           |                          ✓                          |                                                      |    ✓    |
| ESP32-S2 |                          ✓                           |                       &mdash;                       |                       &mdash;                        |    ✓    |
| ESP32-S3 |                          ✓                           |                          ✓                          |                          ✓                           |    ✓    |
| ESP32-C3 |                          ✓                           |                          ✓                          |                          ✓                           |    ✓    |
| ESP32-C2 |                          ✓                           |                          ✓                          |                                                      |    ✓    |
| ESP32-C6 |                          ✓                           |                                                     |                                                      |    ✓    |


## Usage

### Importing

For now this is not available on _crates.io_. Until then you need to import via git, cloning locally, etc.

```toml
[dependencies.esp-wifi]
git = "https://github.com/esp-rs/esp-wifi.git"

# `esp-wifi` is in active development. It is often a good idea to lock it to a specific commit
rev = "c7ca849274cf3d7a08b49c260bb46693c91c85c0"

# A supported chip needs to be specified, as well as specific use-case features 
features = ["esp32s3", "wifi", "esp-now"]
```

### Link configuration

> [!IMPORTANT]
> Make sure to include the rom functions for your target:

```toml
# .cargo/config.toml
rustflags = [
    "-C", "link-arg=-Tlinkall.x",
    "-C", "link-arg=-Trom_functions.x",
]
```
> [!NOTE]
> At time of writing, you will already have the linkall flag if you used `cargo generate`. Generating from a template does not include the `rom_functions` flag.


### Optimization Level

> [!IMPORTANT]
> Link time optimization is not yet recommended for use, please ensure `lto = "off"` is in your `Cargo.toml` for both release and debug profiles.

It is necessary to build with optimization level 2 or 3 since otherwise it might not even be able to connect or advertise.

To make it work also for your debug builds add this to your `Cargo.toml`

```toml
[profile.dev.package.esp-wifi]
opt-level = 3

[profile.dev]
lto = "off"
[profile.release]
lto = "off"

```


### Features

| Feature        | Meaning                                                                                             |
| -------------- | --------------------------------------------------------------------------------------------------- |
| wifi-logs      | logs the WiFi logs from the driver at log level info                                                |
| dump-packets   | dumps packet info at log level info                                                                 |
| utils          | Provide utilities for smoltcp initialization, this is a default feature                             |
| embedded-svc   | Provides a (very limited) implementation of the `embedded-svc` WiFi trait, includes `utils` feature |
| ble            | Enable BLE support                                                                                  |
| wifi           | Enable WiFi support                                                                                 |
| esp-now        | Enable esp-now support                                                                              |
| coex           | Enable coex support                                                                                 |
| mtu-XXX        | Set MTU to XXX, XXX can be 746, 1492, 1500, 1514. Defaults to 1492                                  |
| big-heap       | Reserve more heap memory for the drivers                                                            |
| ipv6           | IPv6 support                                                                                        |
| phy-enable-usb | See _Using Serial-JTAG_ below                                                                       |
| ps-min-modem   | Enable modem sleep                                                                                  |

When using the `dump-packets` feature you can use the extcap in `extras/esp-wifishark` to analyze the frames in Wireshark.
For more information see [extras/esp-wifishark/README.md](extras/esp-wifishark/README.md)

### Serial-JTAG
> [!IMPORTANT]
> On ESP32-C3 / ESP32-S3 when using Serial-JTAG you have to activate the feature `phy-enable-usb`.
> 
> Don't use this feature if your are _not_ using Serial-JTAG since it might reduce WiFi performance.

### Tuning

The defaults used by `esp-wifi` and the examples are rather conservative. It is possible to change a few of the important settings.

See [Tuning](docs/tuning.md) for details

### What works?

- scanning for WiFi access points
- connect to WiFi access point
- providing an HCI interface
- create an open access point

### Notes on ESP32-C2 / ESP32-C3 / ESP32-C6 support

- uses SYSTIMER as the main timer
- doesn't work in direct-boot mode

### Notes on ESP32 / ESP32-S2 / ESP32-S3 support

- The WiFi logs only print the format string - not the actual values.
- The code runs on a single core and might currently not be multi-core safe!

On ESP32 / ESP32-S2 / ESP32-S3 currently TIMG1/TIMER0 is used as the main timer so you can't use it for anything else.
Additionally it uses CCOMPARE0 - so don't touch that, too.

### opt-level for Xtensa targets

Currently your mileage might vary a lot for different opt-levels on Xtensa targets!
If something doesn't work as expected try a different opt-level.


## Examples

To build these ensure you are in the `examples-esp32XXX` directory matching your target as othewise the `config.toml` will not apply

### dhcp

- set SSID and PASSWORD env variable
- gets an ip address via DHCP
- performs an HTTP get request to some "random" server

`cargo run --example dhcp --release --features "embedded-svc,wifi"`

### static_ip

- set SSID and PASSWORD env variable
- set STATIC_IP and GATEWAY_IP env variable (e.g. "192.168.2.191" / "192.168.2.1")
- might be necessary to configure your WiFi access point accordingly
- uses the given static IP
- responds with some HTML content when connecting to port 8080

`cargo run --example static_ip --release --features "embedded-svc,wifi"`

### ble

- starts Bluetooth advertising
- offers one service with three characteristics (one is read/write, one is write only, one is read/write/notify)
- pressing the boot-button on a dev-board will send a notification if it is subscribed
- this uses a toy level BLE stack - might not work with every BLE central device (tested with Android and Windows Bluetooth LE Explorer)

`cargo run --example ble --release --features "ble"` 

**NOTE:** ESP32-S2 doesn't support bluetooth, for ESP32-C6 bluetooth support isn't implemented yet 

### async_ble

- same as `ble` but async

`cargo run --example async_ble --release --features "async,ble"` 

**NOTE:** ESP32-S2 doesn't support bluetooth, for ESP32-C6 bluetooth support isn't implemented yet 

### coex

- set SSID and PASSWORD env variable
- gets an ip address via DHCP
- performs an HTTP get request to some "random" server
- does BLE advertising
- coex support is still somewhat flaky

`cargo run --example coex --release --features "embedded-svc,wifi,ble"`

**NOTE:** Not currently available for the ESP32, ESP32-C2, ESP32-C6 or ESP32-S2

### esp_now

- broadcasts, receives and sends messages via esp-now

`cargo run --example esp_now --release --features "esp-now"`

### embassy_esp_now

- broadcasts, receives and sends messages via esp-now in an async way

`cargo run --example embassy_esp_now --release --features "async,esp-now"`        

### embassy_dhcp

- Read and Write to sockets over WiFi asyncronously using embassy-executor.

`cargo run --example embassy_dhcp --release --features "async,embedded-svc,wifi,embassy-net"`

### access_point

- creates an open access-point with SSID `esp-wifi`
- you can connect to it using a static IP in range 192.168.2.2 .. 192.168.2.255, gateway 192.168.2.1
- open http://192.168.2.1:8080/ in your browser
- on Android you might need to choose _Keep Accesspoint_ when it tells you the WiFi has no internet connection, Chrome might not want to load the URL - you can use a shell and try `curl` and `ping`

`cargo run --example access_point --release --features "embedded-svc,wifi"`

### embassy_access_point

- creates an open access-point with SSID `esp-wifi`
- you can connect to it using a static IP in range 192.168.2.2 .. 192.168.2.255, gateway 192.168.2.1
- open http://192.168.2.1:8080/ in your browser
- on Android you might need to choose _Keep Accesspoint_ when it tells you the WiFi has no internet connection, Chrome might not want to load the URL - you can use a shell and try `curl` and `ping`

`cargo run --example embassy_access_point --release --features "async,embedded-svc,wifi,embassy-net"`


## Directory Structure

- `src/timer-espXXX.rs`: systimer code used for timing and task switching
- `src/preemt/`: a bare minimum RISCV and Xtensa round-robin task scheduler
- `src/compat/`: code needed to emulate enough of an (RT)OS to use the driver
  - `common.rs`: basics like semaphores and recursive mutexes
  - `timer_compat.rs`: code to emulate timer related functionality
- `examples/*.rs`: examples

## Missing / To be done

- lots of refactoring
- make CoEx work on ESP32 (it kind of works when commenting out setting the country in wifi_start, probably some mis-compilation since it then crashes in a totally different code path)
- combined SoftAP/STA mode
- support for non-open SoftAP

## License

Licensed under either of:

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in
the work by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without
any additional terms or conditions.
