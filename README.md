# esp-wifi-sys

This repository hosts the wireless-driver's binaries and low-level bindings.

If you are looking for `esp-wifi` see [esp-radio in esp-hal](https://github.com/esp-rs/esp-hal/tree/main/esp-radio)

The headers and libraries are prepared in [esp-wireless-drivers-3rdparty](https://github.com/esp-rs/esp-wireless-drivers-3rdparty).

Depending on the `lld` version used by the toolchain you might see linker errors like this after copying the libs:

```
rust-lld: error: relocation refers to a symbol in a discarded section: .L0 
            >>> defined in /Users/espressif/action-runner/esp-rs/_work/esp-hal/esp-hal/target/riscv32imac-unknown-none-elf/debug/build/esp-wifi-sys-esp32c61-e592915ee1a41dea/out/libphy.a(phy_api.o)
            >>> referenced by phy_api.o:(.eh_frame+0x12d4) in archive /Users/espressif/action-runner/esp-rs/_work/esp-hal/esp-hal/target/riscv32imac-unknown-none-elf/debug/build/esp-wifi-sys-esp32c61-e592915ee1a41dea/out/libphy.a
```

You can strip the `.eh_frame` segment from these binaries like this, to make it work:
`riscv32-esp-elf-objcopy --remove-section=.eh_frame esp-wifi-sys-esp32c61/libs/libphy.a`


## License

Licensed under either of:

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in
the work by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without
any additional terms or conditions.
