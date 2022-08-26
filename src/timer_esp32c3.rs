use esp32c3_hal as hal;
use esp32c3_hal::interrupt::TrapFrame;
use esp32c3_hal::prelude::*;
use hal::pac;
use hal::pac::Interrupt;
use hal::systimer::{Alarm, Target};

use crate::{binary, preempt::preempt::task_switch};
use log::trace;

pub const TICKS_PER_SECOND: u64 = 16_000_000;

#[cfg(debug_assertions)]
const TIMER_DELAY: fugit::HertzU32 = fugit::HertzU32::from_raw(500);
#[cfg(not(debug_assertions))]
const TIMER_DELAY: fugit::HertzU32 = fugit::HertzU32::from_raw(1_000);

pub fn setup_timer_isr(systimer: Alarm<Target, 0>) {
    let alarm0 = systimer.into_periodic();
    alarm0.set_period(TIMER_DELAY.into());
    alarm0.clear_interrupt();
    alarm0.enable_interrupt();

    esp32c3_hal::interrupt::enable(
        Interrupt::SYSTIMER_TARGET0,
        hal::interrupt::Priority::Priority1,
    )
    .unwrap();
    esp32c3_hal::interrupt::enable(Interrupt::WIFI_MAC, hal::interrupt::Priority::Priority1)
        .unwrap();
    esp32c3_hal::interrupt::enable(Interrupt::RWBT, hal::interrupt::Priority::Priority1).unwrap();
    esp32c3_hal::interrupt::enable(Interrupt::RWBLE, hal::interrupt::Priority::Priority1).unwrap();
    esp32c3_hal::interrupt::enable(Interrupt::BT_BB, hal::interrupt::Priority::Priority1).unwrap();

    unsafe {
        riscv::interrupt::enable();
    }

    while unsafe {
        crate::preempt::preempt::FIRST_SWITCH.load(core::sync::atomic::Ordering::Relaxed)
    } {}
}

#[interrupt]
fn WIFI_MAC() {
    unsafe {
        let (fnc, arg) = crate::wifi::os_adapter::ISR_INTERRUPT_1;

        trace!("interrupt WIFI_MAC {:p} {:p}", fnc, arg);

        if !fnc.is_null() {
            let fnc: fn(*mut binary::c_types::c_void) = core::mem::transmute(fnc);
            fnc(arg);
        }

        trace!("interrupt 1 done");
    };
}

#[interrupt]
fn RWBT() {
    unsafe {
        let intr = &*pac::INTERRUPT_CORE0::ptr();
        intr.cpu_int_clear.write(|w| w.bits(1 << 1));

        let (fnc, arg) = crate::ble::ble_os_adapter_chip_specific::BT_INTERRUPT_FUNCTION5;

        trace!("interrupt RWBT {:p} {:p}", fnc, arg);

        if !fnc.is_null() {
            let fnc: fn(*mut binary::c_types::c_void) = core::mem::transmute(fnc);
            fnc(arg);
        }

        trace!("interrupt 5 done");
    };
}

#[interrupt]
fn RWBLE() {
    unsafe {
        let intr = &*pac::INTERRUPT_CORE0::ptr();
        intr.cpu_int_clear.write(|w| w.bits(1 << 1));

        let (fnc, arg) = crate::ble::ble_os_adapter_chip_specific::BT_INTERRUPT_FUNCTION5;

        trace!("interrupt RWBLE {:p} {:p}", fnc, arg);

        if !fnc.is_null() {
            let fnc: fn(*mut binary::c_types::c_void) = core::mem::transmute(fnc);
            fnc(arg);
        }

        trace!("interrupt 5 done");
    };
}
#[interrupt]
fn BT_BB(_trap_frame: &mut TrapFrame) {
    unsafe {
        let intr = &*pac::INTERRUPT_CORE0::ptr();
        intr.cpu_int_clear.write(|w| w.bits(1 << 1));

        let (fnc, arg) = crate::ble::ble_os_adapter_chip_specific::BT_INTERRUPT_FUNCTION8;

        trace!("interrupt BT_BB {:p} {:p}", fnc, arg);

        if !fnc.is_null() {
            let fnc: fn(*mut binary::c_types::c_void) = core::mem::transmute(fnc);
            fnc(arg);
        }

        trace!("interrupt 8 done");
    };
}

#[interrupt]
fn SYSTIMER_TARGET0(trap_frame: &mut TrapFrame) {
    unsafe {
        // clear the systimer intr
        (*pac::SYSTIMER::ptr())
            .int_clr
            .write(|w| w.target0_int_clr().set_bit());

        task_switch(trap_frame);
    }
}

/// Current systimer count value
/// A tick is 1 / 16_000_000 seconds
pub fn get_systimer_count() -> u64 {
    unsafe {
        let systimer = &(*pac::SYSTIMER::ptr());

        systimer.unit0_op.write(|w| w.bits(1 << 30));

        // wait for value available
        loop {
            let valid = (systimer.unit0_op.read().bits() >> 29) & 1;
            if valid != 0 {
                break;
            }
        }

        let value_lo = systimer.unit0_value_lo.read().bits() as u64;
        let value_hi = (systimer.unit0_value_hi.read().bits() as u64) << 32;

        (value_lo | value_hi) as u64
    }
}
