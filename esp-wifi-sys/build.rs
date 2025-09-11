use std::{
    env,
    fs::{self, File},
    io::Write,
    path::PathBuf,
};

use anyhow::Result;

fn main() -> Result<()> {
    // Put the linker script somewhere the linker can find it
    let out = PathBuf::from(env::var_os("OUT_DIR").unwrap());

    #[cfg(feature = "esp32")]
    {
        copy_libraries(&out)?;
    }
    #[cfg(feature = "esp32c2")]
    {
        copy_libraries(&out)?;
    }
    #[cfg(feature = "esp32c3")]
    {
        copy_libraries(&out)?;
    }
    #[cfg(feature = "esp32c6")]
    {
        copy_libraries(&out)?;
    }
    #[cfg(feature = "esp32h2")]
    {
        copy_libraries(&out)?;
    }
    #[cfg(feature = "esp32s2")]
    {
        copy_libraries(&out)?;
    }
    #[cfg(feature = "esp32s3")]
    {
        copy_libraries(&out)?;
    }

    println!("cargo:rustc-link-search={}", out.display());

    Ok(())
}

fn copy_file(out: &PathBuf, from: &str, to: &str) -> Result<()> {
    let mut file = File::create(out.join(to))?;
    file.write_all(&fs::read(from)?)?;

    Ok(())
}

#[cfg(feature = "esp32")]
fn copy_libraries(out: &PathBuf) -> Result<()> {
    copy_file(out, "libs/esp32/libbtdm_app.a", "libbtdm_app.a")?;
    copy_file(out, "libs/esp32/libcoexist.a", "libcoexist.a")?;
    copy_file(out, "libs/esp32/libcore.a", "libcore.a")?;
    copy_file(out, "libs/esp32/libespnow.a", "libespnow.a")?;
    copy_file(out, "libs/esp32/libmesh.a", "libmesh.a")?;
    copy_file(out, "libs/esp32/libnet80211.a", "libnet80211.a")?;
    copy_file(out, "libs/esp32/libphy.a", "libphy.a")?;
    copy_file(out, "libs/esp32/libpp.a", "libpp.a")?;
    copy_file(out, "libs/esp32/librtc.a", "librtc.a")?;
    copy_file(out, "libs/esp32/libsmartconfig.a", "libsmartconfig.a")?;
    copy_file(out, "libs/esp32/libwapi.a", "libwapi.a")?;
    copy_file(out, "libs/esp32/libwpa_supplicant.a", "libwpa_supplicant.a")?;
    copy_file(out, "libs/esp32/libprintf.a", "libprintf.a")?;
    copy_file(out, "libs/esp32/libregulatory.a", "libregulatory.a")?;

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
    println!("cargo:rustc-link-lib={}", "printf");
    println!("cargo:rustc-link-lib={}", "regulatory");

    Ok(())
}

#[cfg(feature = "esp32c2")]
fn copy_libraries(out: &PathBuf) -> Result<()> {
    copy_file(out, "libs/esp32c2/libble_app.a", "libble_app.a")?;
    copy_file(out, "libs/esp32c2/libbtbb.a", "libbtbb.a")?;
    copy_file(out, "libs/esp32c2/libcoexist.a", "libcoexist.a")?;
    copy_file(out, "libs/esp32c2/libcore.a", "libcore.a")?;
    copy_file(out, "libs/esp32c2/libespnow.a", "libespnow.a")?;
    copy_file(out, "libs/esp32c2/libnet80211.a", "libnet80211.a")?;
    copy_file(out, "libs/esp32c2/libphy.a", "libphy.a")?;
    copy_file(out, "libs/esp32c2/libpp.a", "libpp.a")?;
    copy_file(out, "libs/esp32c2/libsmartconfig.a", "libsmartconfig.a")?;
    copy_file(
        out,
        "libs/esp32c2/libwpa_supplicant.a",
        "libwpa_supplicant.a",
    )?;
    copy_file(out, "libs/esp32c2/libprintf.a", "libprintf.a")?;
    copy_file(out, "libs/esp32c2/libregulatory.a", "libregulatory.a")?;

    println!("cargo:rustc-link-lib={}", "ble_app");
    println!("cargo:rustc-link-lib={}", "btbb");
    println!("cargo:rustc-link-lib={}", "coexist");
    println!("cargo:rustc-link-lib={}", "core");
    println!("cargo:rustc-link-lib={}", "espnow");
    println!("cargo:rustc-link-lib={}", "net80211");
    println!("cargo:rustc-link-lib={}", "phy");
    println!("cargo:rustc-link-lib={}", "pp");
    println!("cargo:rustc-link-lib={}", "smartconfig");
    println!("cargo:rustc-link-lib={}", "wpa_supplicant");
    println!("cargo:rustc-link-lib={}", "printf");
    println!("cargo:rustc-link-lib={}", "regulatory");

    Ok(())
}

#[cfg(feature = "esp32c3")]
fn copy_libraries(out: &PathBuf) -> Result<()> {
    copy_file(out, "libs/esp32c3/libbtbb.a", "libbtbb.a")?;
    copy_file(out, "libs/esp32c3/libbtdm_app.a", "libbtdm_app.a")?;
    copy_file(out, "libs/esp32c3/libcoexist.a", "libcoexist.a")?;
    copy_file(out, "libs/esp32c3/libcore.a", "libcore.a")?;
    copy_file(out, "libs/esp32c3/libespnow.a", "libespnow.a")?;
    copy_file(out, "libs/esp32c3/libmesh.a", "libmesh.a")?;
    copy_file(out, "libs/esp32c3/libnet80211.a", "libnet80211.a")?;
    copy_file(out, "libs/esp32c3/libphy.a", "libphy.a")?;
    copy_file(out, "libs/esp32c3/libpp.a", "libpp.a")?;
    copy_file(out, "libs/esp32c3/libsmartconfig.a", "libsmartconfig.a")?;
    copy_file(out, "libs/esp32c3/libwapi.a", "libwapi.a")?;
    copy_file(
        out,
        "libs/esp32c3/libwpa_supplicant.a",
        "libwpa_supplicant.a",
    )?;
    copy_file(out, "libs/esp32c3/libprintf.a", "libprintf.a")?;
    copy_file(out, "libs/esp32c3/libregulatory.a", "libregulatory.a")?;

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
    println!("cargo:rustc-link-lib={}", "printf");
    println!("cargo:rustc-link-lib={}", "regulatory");

    Ok(())
}

#[cfg(feature = "esp32c6")]
fn copy_libraries(out: &PathBuf) -> Result<()> {
    copy_file(out, "libs/esp32c6/libble_app.a", "libble_app.a")?;
    copy_file(out, "libs/esp32c6/libbtbb.a", "libbtbb.a")?;
    copy_file(out, "libs/esp32c6/libcoexist.a", "libcoexist.a")?;
    copy_file(out, "libs/esp32c6/libcore.a", "libcore.a")?;
    copy_file(out, "libs/esp32c6/libespnow.a", "libespnow.a")?;
    copy_file(out, "libs/esp32c6/libmesh.a", "libmesh.a")?;
    copy_file(out, "libs/esp32c6/libnet80211.a", "libnet80211.a")?;
    copy_file(out, "libs/esp32c6/libphy.a", "libphy.a")?;
    copy_file(out, "libs/esp32c6/libpp.a", "libpp.a")?;
    copy_file(out, "libs/esp32c6/libsmartconfig.a", "libsmartconfig.a")?;
    copy_file(out, "libs/esp32c6/libwapi.a", "libwapi.a")?;
    copy_file(
        out,
        "libs/esp32c6/libwpa_supplicant.a",
        "libwpa_supplicant.a",
    )?;
    copy_file(out, "libs/esp32c6/libprintf.a", "libprintf.a")?;
    copy_file(out, "libs/esp32c6/libregulatory.a", "libregulatory.a")?;

    println!("cargo:rustc-link-lib={}", "ble_app");
    println!("cargo:rustc-link-lib={}", "btbb");
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
    println!("cargo:rustc-link-lib={}", "printf");
    println!("cargo:rustc-link-lib={}", "regulatory");

    Ok(())
}

#[cfg(feature = "esp32h2")]
fn copy_libraries(out: &PathBuf) -> Result<()> {
    copy_file(out, "libs/esp32h2/libble_app.a", "libble_app.a")?;
    copy_file(out, "libs/esp32h2/libbtbb.a", "libbtbb.a")?;
    copy_file(out, "libs/esp32h2/libcoexist.a", "libcoexist.a")?;
    copy_file(out, "libs/esp32h2/libphy.a", "libphy.a")?;
    copy_file(out, "libs/esp32h2/libprintf.a", "libprintf.a")?;

    println!("cargo:rustc-link-lib={}", "ble_app");
    println!("cargo:rustc-link-lib={}", "btbb");
    println!("cargo:rustc-link-lib={}", "coexist");
    println!("cargo:rustc-link-lib={}", "phy");
    println!("cargo:rustc-link-lib={}", "printf");

    Ok(())
}

#[cfg(feature = "esp32s2")]
fn copy_libraries(out: &PathBuf) -> Result<()> {
    copy_file(out, "libs/esp32s2/libcoexist.a", "libcoexist.a")?;
    copy_file(out, "libs/esp32s2/libcore.a", "libcore.a")?;
    copy_file(out, "libs/esp32s2/libespnow.a", "libespnow.a")?;
    copy_file(out, "libs/esp32s2/libmesh.a", "libmesh.a")?;
    copy_file(out, "libs/esp32s2/libnet80211.a", "libnet80211.a")?;
    copy_file(out, "libs/esp32s2/libphy.a", "libphy.a")?;
    copy_file(out, "libs/esp32s2/libpp.a", "libpp.a")?;
    copy_file(out, "libs/esp32s2/libsmartconfig.a", "libsmartconfig.a")?;
    copy_file(out, "libs/esp32s2/libwapi.a", "libwapi.a")?;
    copy_file(
        out,
        "libs/esp32s2/libwpa_supplicant.a",
        "libwpa_supplicant.a",
    )?;
    copy_file(out, "libs/esp32s2/libprintf.a", "libprintf.a")?;
    copy_file(out, "libs/esp32s2/libregulatory.a", "libregulatory.a")?;

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
    println!("cargo:rustc-link-lib={}", "printf");
    println!("cargo:rustc-link-lib={}", "regulatory");

    Ok(())
}

#[cfg(feature = "esp32s3")]
fn copy_libraries(out: &PathBuf) -> Result<()> {
    copy_file(out, "libs/esp32s3/libbtbb.a", "libbtbb.a")?;
    copy_file(out, "libs/esp32s3/libbtdm_app.a", "libbtdm_app.a")?;
    copy_file(out, "libs/esp32s3/libcoexist.a", "libcoexist.a")?;
    copy_file(out, "libs/esp32s3/libcore.a", "libcore.a")?;
    copy_file(out, "libs/esp32s3/libespnow.a", "libespnow.a")?;
    copy_file(out, "libs/esp32s3/libmesh.a", "libmesh.a")?;
    copy_file(out, "libs/esp32s3/libnet80211.a", "libnet80211.a")?;
    copy_file(out, "libs/esp32s3/libphy.a", "libphy.a")?;
    copy_file(out, "libs/esp32s3/libpp.a", "libpp.a")?;
    copy_file(out, "libs/esp32s3/libsmartconfig.a", "libsmartconfig.a")?;
    copy_file(out, "libs/esp32s3/libwapi.a", "libwapi.a")?;
    copy_file(
        out,
        "libs/esp32s3/libwpa_supplicant.a",
        "libwpa_supplicant.a",
    )?;
    copy_file(out, "libs/esp32s3/libprintf.a", "libprintf.a")?;
    copy_file(out, "libs/esp32s3/libregulatory.a", "libregulatory.a")?;

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
    println!("cargo:rustc-link-lib={}", "printf");
    println!("cargo:rustc-link-lib={}", "regulatory");

    Ok(())
}
