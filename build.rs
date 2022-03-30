use std::env;
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;

#[cfg(feature = "esp32c3")]
fn main() {
    // Put the linker script somewhere the linker can find it
    let out = &PathBuf::from(env::var_os("OUT_DIR").unwrap());
    File::create(out.join("esp32c3_rom_functions.x"))
        .unwrap()
        .write_all(include_bytes!("esp32c3_rom_functions.x"))
        .unwrap();

    File::create(out.join("esp32c3-wifi-link.x"))
        .unwrap()
        .write_all(include_bytes!("esp32c3-wifi-link.x"))
        .unwrap();

    println!("cargo:rustc-link-search={}", out.display());

    // Only re-run the build script when memory.x is changed,
    // instead of when any part of the source code changes.
    println!("cargo:rerun-if-changed=memory.x");
}

#[cfg(feature = "esp32")]
fn main() {
    // Put the linker script somewhere the linker can find it
    let out = &PathBuf::from(env::var_os("OUT_DIR").unwrap());
    File::create(out.join("esp32_rom_functions.x"))
        .unwrap()
        .write_all(include_bytes!("esp32_rom_functions.x"))
        .unwrap();

    File::create(out.join("esp32-wifi-link.x"))
        .unwrap()
        .write_all(include_bytes!("esp32-wifi-link.x"))
        .unwrap();

    println!("cargo:rustc-link-search={}", out.display());

    // Only re-run the build script when memory.x is changed,
    // instead of when any part of the source code changes.
    println!("cargo:rerun-if-changed=memory.x");
}

#[cfg(not(any(feature = "esp32c3", feature = "esp32")))]
fn main() {
    panic!("Select a chip via it's cargo feature");
}
