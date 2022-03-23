#![no_std]
#![feature(c_variadic)]

pub mod binary;
pub mod compat;
pub mod log;
pub mod preempt;
pub mod timer;
pub mod wifi;

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
