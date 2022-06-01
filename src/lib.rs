#![no_std]
#![cfg_attr(target_arch = "xtensa", feature(asm_experimental_arch))]
#![feature(c_variadic)]
#![feature(layout_for_ptr)]

#[doc(hidden)]
pub mod binary;

#[doc(hidden)]
pub mod compat;

#[doc(hidden)]
pub mod preempt;

#[doc(hidden)]
#[cfg_attr(feature = "esp32", path = "timer_esp32.rs")]
#[cfg_attr(feature = "esp32c3", path = "timer_esp32c3.rs")]
pub mod timer;

pub mod wifi;

pub mod ble;

#[doc(hidden)]
pub mod tasks;

pub(crate) mod memory_fence;

pub use critical_section;
use timer::{get_systimer_count, TICKS_PER_SECOND};

#[cfg(feature = "embedded-svc")]
pub mod wifi_interface;

pub fn current_millis() -> u64 {
    get_systimer_count() / TICKS_PER_SECOND
}

// ---------------------------------------------------------------------------
// Cursed...

pub(crate) static ALLOCATOR: esp_alloc::EspHeap = esp_alloc::EspHeap::empty();

pub fn init_heap() {
    use core::mem::MaybeUninit;

    const HEAP_SIZE: usize = 4 * 1024;
    static mut HEAP: [MaybeUninit<u8>; HEAP_SIZE] = [MaybeUninit::uninit(); HEAP_SIZE];

    unsafe {
        ALLOCATOR.init(HEAP.as_ptr() as usize, HEAP_SIZE);
    }
}
