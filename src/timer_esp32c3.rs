use esp32c3_hal as hal;
use esp32c3_hal::interrupt::TrapFrame;
use esp32c3_hal::pac::INTERRUPT_CORE0;
use esp32c3_hal::pac::SYSTIMER;

use crate::{binary, preempt::preempt::task_switch};
use log::trace;

pub const TICKS_PER_SECOND: u64 = 16_000_000;

#[cfg(debug_assertions)]
const TIMER_DELAY: u32 = 8_000u32;
#[cfg(not(debug_assertions))]
const TIMER_DELAY: u32 = 500u32;

pub fn setup_timer_isr(systimer: &mut SYSTIMER, interrupt_core0: &mut INTERRUPT_CORE0) {
    // set systimer to 0
    systimer.unit0_load_lo.write(|w| unsafe { w.bits(0) });
    systimer.unit0_load_hi.write(|w| unsafe { w.bits(0) });
    systimer.unit0_load.write(|w| unsafe { w.bits(1) });

    // PERIOD_MODE + PERIOD
    systimer
        .target0_conf
        .write(|w| unsafe { w.bits((1 << 30) | TIMER_DELAY) });
    // LOAD CONF VALUE
    systimer.comp0_load.write(|w| unsafe { w.bits(1) });
    // set SYSTIMER_TARGET0_WORK_EN + UNIT0_WORK_EN
    systimer
        .conf
        .write(|w| unsafe { w.bits(1 << 24 | 1 << 30) });

    systimer.int_clr.write(|w| unsafe { w.bits(1 << 0) });

    // TARGET0 INT ENA
    systimer.int_ena.write(|w| unsafe { w.bits(1 << 0) });

    interrupt_core0
        .systimer_target0_int_map
        .write(|w| unsafe { w.bits(10) });
    interrupt_core0
        .cpu_int_pri_10
        .write(|w| unsafe { w.bits(1) }); // PRIO = 1
    interrupt_core0
        .cpu_int_enable
        .write(|w| unsafe { w.bits(1 << 10) }); // ENABLE INT 10

    unsafe {
        riscv::interrupt::enable();
    }

    while unsafe {
        crate::preempt::preempt::FIRST_SWITCH.load(core::sync::atomic::Ordering::Relaxed)
    } {}
}

#[no_mangle]
pub fn interrupt1(_trap_frame: &mut TrapFrame) {
    unsafe {
        let intr = &*hal::pac::INTERRUPT_CORE0::ptr();
        intr.cpu_int_clear.write(|w| w.bits(1 << 1));

        let (fnc, arg) = crate::wifi::os_adapter::ISR_INTERRUPT_1;

        trace!("interrupt 1 {:p} {:p}", fnc, arg);

        if !fnc.is_null() {
            let fnc: fn(*mut binary::c_types::c_void) = core::mem::transmute(fnc);
            fnc(arg);
        }

        trace!("interrupt 1 done");
    };
}

#[no_mangle]
pub fn interrupt10(trap_frame: &mut TrapFrame) {
    unsafe {
        // clear the systimer intr
        (*hal::pac::SYSTIMER::ptr())
            .int_clr
            .write(|w| w.bits(1 << 0));

        task_switch(trap_frame);
    }
}

/// Current systimer count value
/// A tick is 1 / 16_000_000 seconds
pub fn get_systimer_count() -> u64 {
    unsafe {
        let systimer = &(*hal::pac::SYSTIMER::ptr());

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
