use std::env;
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;

#[cfg(feature = "esp32c3")]
fn main() {
    // Put the linker script somewhere the linker can find it
    let out = &PathBuf::from(env::var_os("OUT_DIR").unwrap());

    copy(
        out,
        include_bytes!("ld/esp32c3/rom_functions.x"),
        "esp32c3_rom_functions.x",
    );

    copy(out, include_bytes!("libs/esp32c3/libbtbb.a"), "libbtbb.a");
    copy(
        out,
        include_bytes!("libs/esp32c3/libbtdm_app.a"),
        "libbtdm_app.a",
    );
    copy(
        out,
        include_bytes!("libs/esp32c3/libcoexist.a"),
        "libcoexist.a",
    );
    copy(out, include_bytes!("libs/esp32c3/libcore.a"), "libcore.a");
    copy(
        out,
        include_bytes!("libs/esp32c3/libespnow.a"),
        "libespnow.a",
    );
    copy(out, include_bytes!("libs/esp32c3/libmesh.a"), "libmesh.a");
    copy(
        out,
        include_bytes!("libs/esp32c3/libnet80211.a"),
        "libnet80211.a",
    );
    copy(out, include_bytes!("libs/esp32c3/libphy.a"), "libphy.a");
    copy(out, include_bytes!("libs/esp32c3/libpp.a"), "libpp.a");
    copy(
        out,
        include_bytes!("libs/esp32c3/libsmartconfig.a"),
        "libsmartconfig.a",
    );
    copy(out, include_bytes!("libs/esp32c3/libwapi.a"), "libwapi.a");
    copy(
        out,
        include_bytes!("libs/esp32c3/libwpa_supplicant.a"),
        "libwpa_supplicant.a",
    );

    println!("cargo:rustc-link-lib={}", "btbb");
    println!("cargo:rustc-link-lib={}", "btdm_app");
    println!("cargo:rustc-link-lib={}", "coexist");
    println!("cargo:rustc-link-lib={}", "core");
    println!("cargo:rustc-link-lib={}", "espnow");
    println!("cargo:rustc-link-lib={}", "mesh");
    println!("cargo:rustc-link-lib={}", "net80211");
    println!("cargo:rustc-link-lib={}", "phy");
    println!("cargo:rustc-link-lib={}", "pp");
    println!("cargo:rustc-link-lib={}", "smartconfig");
    println!("cargo:rustc-link-lib={}", "wapi");
    println!("cargo:rustc-link-lib={}", "wpa_supplicant");

    println!("cargo:rustc-link-search={}", out.display());

    // Only re-run the build script when memory.x is changed,
    // instead of when any part of the source code changes.
    println!("cargo:rerun-if-changed=memory.x");
}

#[cfg(feature = "esp32")]
fn main() {
    // Put the linker script somewhere the linker can find it
    let out = &PathBuf::from(env::var_os("OUT_DIR").unwrap());

    copy(
        out,
        include_bytes!("ld/esp32/rom_functions.x"),
        "esp32_rom_functions.x",
    );

    copy(
        out,
        include_bytes!("libs/esp32/libbtdm_app.a"),
        "libbtdm_app.a",
    );
    copy(
        out,
        include_bytes!("libs/esp32/libcoexist.a"),
        "libcoexist.a",
    );
    copy(out, include_bytes!("libs/esp32/libcore.a"), "libcore.a");
    copy(out, include_bytes!("libs/esp32/libespnow.a"), "libespnow.a");
    copy(out, include_bytes!("libs/esp32/libmesh.a"), "libmesh.a");
    copy(
        out,
        include_bytes!("libs/esp32/libnet80211.a"),
        "libnet80211.a",
    );
    copy(out, include_bytes!("libs/esp32/libphy.a"), "libphy.a");
    copy(out, include_bytes!("libs/esp32/libpp.a"), "libpp.a");
    copy(out, include_bytes!("libs/esp32/librtc.a"), "librtc.a");
    copy(
        out,
        include_bytes!("libs/esp32/libsmartconfig.a"),
        "libsmartconfig.a",
    );
    copy(out, include_bytes!("libs/esp32/libwapi.a"), "libwapi.a");
    copy(
        out,
        include_bytes!("libs/esp32/libwpa_supplicant.a"),
        "libwpa_supplicant.a",
    );

    println!("cargo:rustc-link-lib={}", "btdm_app");
    println!("cargo:rustc-link-lib={}", "coexist");
    println!("cargo:rustc-link-lib={}", "core");
    println!("cargo:rustc-link-lib={}", "espnow");
    println!("cargo:rustc-link-lib={}", "mesh");
    println!("cargo:rustc-link-lib={}", "net80211");
    println!("cargo:rustc-link-lib={}", "phy");
    println!("cargo:rustc-link-lib={}", "pp");
    println!("cargo:rustc-link-lib={}", "rtc");
    println!("cargo:rustc-link-lib={}", "smartconfig");
    println!("cargo:rustc-link-lib={}", "wapi");
    println!("cargo:rustc-link-lib={}", "wpa_supplicant");

    println!("cargo:rustc-link-search={}", out.display());

    // Only re-run the build script when memory.x is changed,
    // instead of when any part of the source code changes.
    println!("cargo:rerun-if-changed=memory.x");
}

fn copy(path: &PathBuf, data: &[u8], name: &str) {
    File::create(path.join(name))
        .unwrap()
        .write_all(data)
        .unwrap();
}

#[cfg(not(any(feature = "esp32c3", feature = "esp32")))]
fn main() {
    panic!("Select a chip via it's cargo feature");
}
