#![no_std]
#![cfg_attr(target_arch = "xtensa", feature(asm_experimental_arch))]
#![feature(c_variadic)]
#![feature(layout_for_ptr)]

use core::cell::RefCell;
use core::mem::MaybeUninit;

use critical_section::Mutex;
#[cfg(feature = "esp32")]
use esp32_hal as hal;
#[cfg(feature = "esp32c3")]
use esp32c3_hal as hal;
#[cfg(feature = "esp32c3")]
use esp32c3_hal::systimer::{Alarm, Target};
#[cfg(feature = "esp32s2")]
use esp32s2_hal as hal;
#[cfg(feature = "esp32s3")]
use esp32s3_hal as hal;

use fugit::MegahertzU32;
use hal::clock::Clocks;
use linked_list_allocator::Heap;

use crate::common_adapter::init_rng;
use crate::tasks::init_tasks;
use crate::timer::setup_timer_isr;

#[doc(hidden)]
pub mod binary;

#[doc(hidden)]
pub mod compat;

#[doc(hidden)]
pub mod preempt;

#[doc(hidden)]
#[cfg_attr(feature = "esp32", path = "timer_esp32.rs")]
#[cfg_attr(feature = "esp32c3", path = "timer_esp32c3.rs")]
#[cfg_attr(feature = "esp32s3", path = "timer_esp32s3.rs")]
#[cfg_attr(feature = "esp32s2", path = "timer_esp32s2.rs")]
pub mod timer;

#[cfg(feature = "wifi")]
pub mod wifi;

#[cfg(feature = "ble")]
pub mod ble;

pub(crate) mod common_adapter;

#[doc(hidden)]
pub mod tasks;

pub(crate) mod memory_fence;

pub use critical_section;
use timer::{get_systimer_count, TICKS_PER_SECOND};

#[cfg(all(feature = "embedded-svc", feature = "wifi"))]
pub mod wifi_interface;

#[cfg(feature = "esp32c3")]
use esp32c3_hal::interrupt;

pub fn current_millis() -> u64 {
    get_systimer_count() / (TICKS_PER_SECOND / 1000)
}

#[cfg(not(coex))]
const HEAP_SIZE: usize = 64 * 1024;

#[cfg(coex)]
const HEAP_SIZE: usize = 96 * 1024;

static mut HEAP_DATA: [MaybeUninit<u8>; HEAP_SIZE] = [MaybeUninit::uninit(); HEAP_SIZE];

pub(crate) static HEAP: Mutex<RefCell<Heap>> = Mutex::new(RefCell::new(Heap::empty()));

pub fn init_heap() {
    critical_section::with(|cs| {
        HEAP.borrow(cs)
            .borrow_mut()
            .init_from_slice(unsafe { &mut HEAP_DATA })
    });
}

#[cfg(feature = "esp32c3")]
/// Initialize for using WiFi / BLE
/// This will initialize internals and also initialize WiFi and BLE
pub fn initialize(
    systimer: Alarm<Target, 0>,
    rng: hal::Rng,
    clocks: &Clocks,
) -> Result<(), InitializationError> {
    if clocks.cpu_clock != MegahertzU32::MHz(160) {
        return Err(InitializationError::WrongClockConfig);
    }

    init_rng(rng);
    init_tasks();
    setup_timer_isr(systimer);
    wifi_set_log_verbose();
    init_clocks();
    init_buffer();

    #[cfg(coex)]
    {
        let res = crate::wifi::coex_initialize();
        if res != 0 {
            return Err(InitializationError::General(res));
        }
    }

    #[cfg(feature = "wifi")]
    {
        log::debug!("wifi init");
        // wifi init
        let res = crate::wifi::wifi_init();
        if res != 0 {
            return Err(InitializationError::General(res));
        }
        let res = crate::wifi::wifi_start();
        if res != 0 {
            return Err(InitializationError::General(res));
        }
    }

    #[cfg(feature = "ble")]
    {
        // ble init
        // for some reason things don't work when initializing things the other way around
        // while the original implementation in NuttX does it like that
        log::debug!("ble init");
        crate::ble::ble_init();
    }

    Ok(())
}

#[derive(Debug, Clone, Copy)]
pub enum InitializationError {
    General(i32),
    WrongClockConfig,
}

#[cfg(any(feature = "esp32", feature = "esp32s3", feature = "esp32s2"))]
/// Initialize for using WiFi / BLE
/// This will initialize internals and also initialize WiFi and BLE
pub fn initialize(
    timg1_timer0: hal::timer::Timer<hal::timer::Timer0<hal::pac::TIMG1>>,
    rng: hal::Rng,
    clocks: &Clocks,
) -> Result<(), InitializationError> {
    if clocks.cpu_clock != MegahertzU32::MHz(240) {
        return Err(InitializationError::WrongClockConfig);
    }

    init_rng(rng);
    init_tasks();
    setup_timer_isr(timg1_timer0);
    wifi_set_log_verbose();
    init_clocks();
    init_buffer();

    #[cfg(coex)]
    {
        let res = crate::wifi::coex_initialize();
        if res != 0 {
            return Err(InitializationError::General(res));
        }
    }

    #[cfg(feature = "wifi")]
    {
        log::debug!("wifi init");
        let res = crate::wifi::wifi_init();
        if res != 0 {
            return Err(InitializationError::General(res));
        }
        let res = crate::wifi::wifi_start();
        if res != 0 {
            return Err(InitializationError::General(res));
        }
    }

    #[cfg(feature = "ble")]
    {
        // ble init
        // for some reason things don't work when initializing things the other way around
        // while the original implementation in NuttX does it like that
        log::debug!("ble init");
        crate::ble::ble_init();
    }

    Ok(())
}

pub fn wifi_set_log_verbose() {
    #[cfg(feature = "wifi_logs")]
    unsafe {
        use crate::binary::include::{esp_wifi_internal_set_log_level, wifi_log_level_t};

        let level: wifi_log_level_t = crate::binary::include::wifi_log_level_t_WIFI_LOG_VERBOSE;
        esp_wifi_internal_set_log_level(level);
    }
}

pub fn init_buffer() {
    // nothing anymore for now
}

pub fn init_clocks() {
    crate::common_adapter::chip_specific::init_clocks();
}
