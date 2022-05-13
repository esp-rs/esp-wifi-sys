#![no_std]
#![feature(c_variadic)]
#![cfg_attr(target_arch = "xtensa", feature(asm_experimental_arch))]
#![feature(alloc_error_handler)]

#[doc(hidden)]
pub mod binary;

#[doc(hidden)]
pub mod compat;

#[doc(hidden)]
pub mod preempt;

#[doc(hidden)]
#[cfg_attr(feature = "esp32c3", path = "timer_esp32c3.rs")]
#[cfg_attr(feature = "esp32", path = "timer_esp32.rs")]
pub mod timer;

pub mod wifi;

#[cfg(feature = "esp32c3")]
pub mod ble;

#[doc(hidden)]
pub mod tasks;

pub use critical_section;
use timer::{get_systimer_count, TICKS_PER_SECOND};

#[cfg(feature = "embedded-svc")]
pub mod wifi_interface;

pub fn current_millis() -> u64 {
    get_systimer_count() / TICKS_PER_SECOND
}
