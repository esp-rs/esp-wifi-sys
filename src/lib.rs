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

#[cfg(feature = "esp32c3")]
use esp32c3_hal::interrupt;

pub fn current_millis() -> u64 {
    (get_systimer_count() / TICKS_PER_SECOND) * 1000
}

// TODO: should the below code live somewhere else, in its own module maybe? Or is it fine here?

#[global_allocator]
pub(crate) static ALLOCATOR: esp_alloc::EspHeap = esp_alloc::EspHeap::empty();

pub fn init_heap() {
    const HEAP_SIZE: usize = 64 * 1024;
    extern "C" {
        static mut _heap_start: u32;
        //static mut _heap_end: u32; // XXX we don't have it on ESP32-C3 currently
    }

    unsafe {
        let heap_start = &_heap_start as *const _ as usize;

        //let heap_end = &_heap_end as *const _ as usize;
        //assert!(heap_end - heap_start > HEAP_SIZE, "Not enough available heap memory.");

        ALLOCATOR.init(heap_start as *mut u8, HEAP_SIZE);
    }
}
