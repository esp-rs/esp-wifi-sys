# Bare-metal Wi-Fi and Bluetooth LE for ESP32XX

## [esp-wifi](./esp-wifi)

High-level API for using Wi-Fi and Bluetooth LE.

## [esp-wifi-sys](./esp-wifi-sys)

Low-level unsafe bindings for the binary blobs required by the Wi-Fi/Bluetooth LE radio.

## [xtask](./xtask)

Task automation using the [cargo-xtask] convention, used for generating the `esp-wifi-sys` bindings using [bindgen].

To run the task:

```bash
cargo xtask
```

[cargo-xtask]: https://github.com/matklad/cargo-xtask
[bindgen]: https://github.com/rust-lang/rust-bindgen

## License

Licensed under either of:

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in
the work by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without
any additional terms or conditions.
