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
pub mod print;

#[doc(hidden)]
#[cfg_attr(feature = "esp32c3", path = "timer_esp32c3.rs")]
#[cfg_attr(feature = "esp32", path = "timer_esp32.rs")]
pub mod timer;
pub mod wifi;

#[doc(hidden)]
pub mod tasks;

pub(crate) mod memory_fence;

pub use critical_section;

#[cfg(feature = "allocator")]
pub mod allocator;

#[cfg(feature = "embedded-svc")]
pub mod wifi_interface;

extern "C" {
    // ROM functions, see esp32c3-link.x
    pub fn uart_tx_one_char(byte: u8) -> i32;
}
pub struct Uart;

impl core::fmt::Write for Uart {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        Ok(for &b in s.as_bytes() {
            unsafe { uart_tx_one_char(b) };
        })
    }
}
