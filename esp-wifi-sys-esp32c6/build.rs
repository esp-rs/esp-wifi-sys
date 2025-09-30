use std::{
    env,
    fs::{self, File},
    io::Write,
    path::{Path, PathBuf},
};

use anyhow::Result;

fn main() -> Result<()> {
    // Put the linker script somewhere the linker can find it
    let out = PathBuf::from(env::var_os("OUT_DIR").unwrap());

    let libs = [
        "ble_app",
        "btbb",
        "coexist",
        "core",
        "espnow",
        "mesh",
        "net80211",
        "phy",
        "pp",
        "smartconfig",
        "wapi",
        "wpa_supplicant",
        "printf",
        "regulatory",
    ];

    for lib in libs {
        copy_file(
            &out,
            &format!("libs/lib{}.a", lib),
            &format!("lib{}.a", lib),
        )?;
        println!("cargo:rustc-link-lib={lib}");
    }

    println!("cargo:rustc-link-search={}", out.display());

    Ok(())
}

fn copy_file(out: &Path, from: &str, to: &str) -> Result<()> {
    let mut file = File::create(out.join(to))?;
    file.write_all(&fs::read(from)?)?;

    Ok(())
}
