#![no_std]
#![doc(html_logo_url = "https://avatars.githubusercontent.com/u/46717278")]
#![cfg_attr(feature = "binary-logs", feature(c_variadic))]

pub mod c_types;
mod fmt;

#[allow(improper_ctypes)]
#[cfg_attr(feature = "esp32", path = "include/esp32.rs")]
#[cfg_attr(feature = "esp32c2", path = "include/esp32c2.rs")]
#[cfg_attr(feature = "esp32c3", path = "include/esp32c3.rs")]
#[cfg_attr(feature = "esp32c6", path = "include/esp32c6.rs")]
#[cfg_attr(feature = "esp32h2", path = "include/esp32h2.rs")]
#[cfg_attr(feature = "esp32s2", path = "include/esp32s2.rs")]
#[cfg_attr(feature = "esp32s3", path = "include/esp32s3.rs")]
pub mod include;

#[cfg(feature = "binary-logs")]
pub mod log {
    #[no_mangle]
    pub unsafe extern "C" fn rtc_printf(s: *const u8, args: ...) {
        syslog(0, s, args);
    }

    #[no_mangle]
    pub unsafe extern "C" fn phy_printf(s: *const u8, args: ...) {
        syslog(0, s, args);
    }

    #[no_mangle]
    pub unsafe extern "C" fn coexist_printf(s: *const u8, args: ...) {
        syslog(0, s, args);
    }

    #[no_mangle]
    pub unsafe extern "C" fn net80211_printf(s: *const u8, args: ...) {
        syslog(0, s, args);
    }

    #[no_mangle]
    pub unsafe extern "C" fn pp_printf(s: *const u8, args: ...) {
        syslog(0, s, args);
    }

    #[no_mangle]
    pub unsafe extern "C" fn syslog(_priority: u32, _format: *const u8, _args: ...) {
        #[allow(clashing_extern_declarations)]
        extern "C" {
            fn vsnprintf(buffer: *mut u8, len: usize, fmt: *const u8, args: ...);
        }

        let mut buf = [0u8; 512];
        vsnprintf(&mut buf as *mut u8, 512, _format, _args);
        let res_str = core::ffi::CStr::from_ptr(&buf as *const _ as *const i8);
        info!("{}", res_str.to_str().unwrap());
    }
}

#[cfg(not(feature = "binary-logs"))]
pub mod log {
    #[no_mangle]
    pub unsafe extern "C" fn rtc_printf(_s: *const u8, _args: *const ()) {}

    #[no_mangle]
    pub unsafe extern "C" fn phy_printf(_s: *const u8, _args: *const ()) {}

    #[no_mangle]
    pub unsafe extern "C" fn coexist_printf(_s: *const u8, _args: *const ()) {}

    #[no_mangle]
    pub unsafe extern "C" fn net80211_printf(_s: *const u8, _args: *const ()) {}

    #[no_mangle]
    pub unsafe extern "C" fn pp_printf(_s: *const u8, _args: *const ()) {}

    #[no_mangle]
    pub unsafe extern "C" fn syslog(_priority: u32, _format: *const u8, _args: *const ()) {}
}
