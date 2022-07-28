use core::cell::RefCell;

use atomic_polyfill::{AtomicU64, Ordering};
use esp32_hal::{
    interrupt,
    pac::{self, TIMG1},
    prelude::_embedded_hal_timer_CountDown,
    timer::{Timer, Timer0},
};
use log::trace;
use xtensa_lx::mutex::{Mutex, SpinLockMutex};
use xtensa_lx_rt::exception::Context;

use crate::preempt::preempt::task_switch;
use esp32_hal::macros::interrupt;

pub const TICKS_PER_SECOND: u64 = 40_000_000;

#[cfg(debug_assertions)]
const TIMER_DELAY: fugit::MicrosDurationU64 = fugit::MicrosDurationU64::millis(13);
#[cfg(not(debug_assertions))]
const TIMER_DELAY: fugit::MicrosDurationU64 = fugit::MicrosDurationU64::millis(3);

static mut TIMER1: SpinLockMutex<RefCell<Option<Timer<Timer0<TIMG1>>>>> =
    SpinLockMutex::new(RefCell::new(None));

static mut TIME: AtomicU64 = AtomicU64::new(0);

pub fn get_systimer_count() -> u64 {
    unsafe { TIME.load(Ordering::Relaxed) + read_timer_value() }
}

#[inline(always)]
fn read_timer_value() -> u64 {
    let value = xtensa_lx::timer::get_cycle_count() as u64;
    value * 40_000_000 / 240_000_000
}

pub fn setup_timer_isr(timg1_timer0: Timer<Timer0<TIMG1>>) {
    let mut timer1 = timg1_timer0;
    interrupt::enable(pac::Interrupt::TG1_T0_LEVEL, interrupt::Priority::Priority2).unwrap();
    interrupt::enable(pac::Interrupt::WIFI_MAC, interrupt::Priority::Priority1).unwrap();
    interrupt::enable(pac::Interrupt::RWBT, interrupt::Priority::Priority1).unwrap();
    interrupt::enable(pac::Interrupt::RWBLE, interrupt::Priority::Priority1).unwrap();
    interrupt::enable(pac::Interrupt::BT_BB, interrupt::Priority::Priority1).unwrap();

    timer1.listen();
    timer1.start(TIMER_DELAY.convert());
    unsafe {
        (&TIMER1).lock(|data| (*data).replace(Some(timer1)));
    }

    xtensa_lx::timer::set_ccompare0(0xffffffff);

    unsafe {
        xtensa_lx::interrupt::disable();
        xtensa_lx::interrupt::enable_mask(
            1 << 6
                | xtensa_lx_rt::interrupt::CpuInterruptLevel::Level2.mask()
                | xtensa_lx_rt::interrupt::CpuInterruptLevel::Level6.mask(),
        );
    }

    while unsafe {
        crate::preempt::preempt::FIRST_SWITCH.load(core::sync::atomic::Ordering::Relaxed)
    } {}
}

#[allow(non_snake_case)]
#[no_mangle]
fn Software0(_level: u32) {
    unsafe {
        let (fnc, arg) = crate::ble::ble_os_adapter_chip_specific::ISR_INTERRUPT_7;
        trace!("interrupt Software0 {:p} {:p}", fnc, arg);

        if !fnc.is_null() {
            let fnc: fn(*mut crate::binary::c_types::c_void) = core::mem::transmute(fnc);
            fnc(arg);
        }
    }
}

#[allow(non_snake_case)]
#[no_mangle]
fn Timer0(_level: u32) {
    unsafe {
        TIME.fetch_add(0x1_0000_0000 * 40_000_000 / 240_000_000, Ordering::Relaxed);
    }

    xtensa_lx::timer::set_ccompare0(0xffffffff);
}

#[interrupt]
fn WIFI_MAC() {
    unsafe {
        let (fnc, arg) = crate::wifi::os_adapter::ISR_INTERRUPT_1;
        trace!("interrupt WIFI_MAC {:p} {:p}", fnc, arg);

        if !fnc.is_null() {
            let fnc: fn(*mut crate::binary::c_types::c_void) = core::mem::transmute(fnc);
            fnc(arg);
        }
    }
}

#[interrupt]
fn RWBT() {
    unsafe {
        let (fnc, arg) = crate::ble::ble_os_adapter_chip_specific::ISR_INTERRUPT_5;
        trace!("interrupt RWBT {:p} {:p}", fnc, arg);

        if !fnc.is_null() {
            let fnc: fn(*mut crate::binary::c_types::c_void) = core::mem::transmute(fnc);
            fnc(arg);
        }
    }
}

#[interrupt]
fn RWBLE() {
    unsafe {
        let (fnc, arg) = crate::ble::ble_os_adapter_chip_specific::ISR_INTERRUPT_5;
        trace!("interrupt RWBLE {:p} {:p}", fnc, arg);

        if !fnc.is_null() {
            let fnc: fn(*mut crate::binary::c_types::c_void) = core::mem::transmute(fnc);
            fnc(arg);
        }
    }
}

#[interrupt]
fn BT_BB() {
    unsafe {
        let (fnc, arg) = crate::ble::ble_os_adapter_chip_specific::ISR_INTERRUPT_8;
        trace!("interrupt BT_BB {:p} {:p}", fnc, arg);

        if !fnc.is_null() {
            let fnc: fn(*mut crate::binary::c_types::c_void) = core::mem::transmute(fnc);
            fnc(arg);
        }
    }
}

#[interrupt]
fn TG1_T0_LEVEL(context: &mut Context) {
    task_switch(context);

    unsafe {
        (&TIMER1).lock(|data| {
            crate::memory_fence::memory_fence();

            let mut timer1 = data.borrow_mut();
            let timer1 = timer1.as_mut().unwrap();

            timer1.clear_interrupt();
            timer1.start(TIMER_DELAY.convert());
        });
    }
}
