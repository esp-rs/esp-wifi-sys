use std::{env, path::PathBuf};

fn main() {
    // Put the linker script somewhere the linker can find it
    let out = PathBuf::from(env::var_os("OUT_DIR").unwrap());

    let wifi = env::var_os("CARGO_FEATURE_WIFI").is_some();
    let bt = env::var_os("CARGO_FEATURE_BT").is_some();
    let ieee802154 = env::var_os("CARGO_FEATURE_IEEE802154").is_some();

    let libs = [
        ("ble_app", bt),
        ("btbb", bt || ieee802154),
        ("coexist", wifi || bt || ieee802154),
        ("core", wifi),
        ("espnow", wifi),
        ("mesh", wifi),
        ("net80211", wifi),
        ("phy", true),
        ("pp", wifi),
        ("smartconfig", wifi),
        ("wapi", wifi),
        ("wpa_supplicant", wifi),
        ("printf", true),
        ("regulatory", wifi),
    ];

    for (lib, _) in libs.into_iter().filter(|(_, enabled)| *enabled) {
        std::fs::copy(
            format!("libs/lib{}.a", lib),
            out.join(format!("lib{}.a", lib)),
        )
        .unwrap_or_else(|e| panic!("Failed to copy the {lib} library: {e}"));
        println!("cargo:rustc-link-lib={lib}");
    }

    println!("cargo:rustc-link-search={}", out.display());
}
