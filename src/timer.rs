use hal::{interrupt::TrapFrame, pac::Peripherals};

use crate::{
    binary,
    compat::{
        self,
        queue::SimpleQueue,
        timer_compat::{Timer, TIMERS},
    },
    preempt::{task_create, task_switch},
    trace,
    wifi::send_data_if_needed,
    Uart,
};
use core::fmt::Write;

pub fn init_intr11(peripherals: &Peripherals) {
    // esp32c3_wl_init sets an interrupt handler - not clear who generates the interrupt etc.
    // seems unused - check and remove

    peripherals
        .INTERRUPT_CORE0
        .cpu_int_pri_11
        .write(|w| unsafe { w.bits(2) }); // PRIO = 2
    peripherals
        .INTERRUPT_CORE0
        .cpu_int_enable
        .modify(|r, w| unsafe { w.bits(r.bits() | (1 << 11)) }); // ENABLE INT 11

    peripherals
        .SYSTEM
        .cpu_intr_from_cpu_0
        .write(|w| unsafe { w.bits(1) });
}

pub fn setup_timer_isr(peripherals: &Peripherals) {
    // set systimer to 0
    peripherals
        .SYSTIMER
        .unit0_load_lo
        .write(|w| unsafe { w.bits(0) });
    peripherals
        .SYSTIMER
        .unit0_load_hi
        .write(|w| unsafe { w.bits(0) });
    peripherals
        .SYSTIMER
        .unit0_load
        .write(|w| unsafe { w.bits(1) });

    // PERIOD_MODE + PERIOD
    peripherals
        .SYSTIMER
        .target0_conf
        .write(|w| unsafe { w.bits((1 << 30) | 20_000) });
    // LOAD CONF VALUE
    peripherals
        .SYSTIMER
        .comp0_load
        .write(|w| unsafe { w.bits(1) });
    // set SYSTIMER_TARGET0_WORK_EN + UNIT0_WORK_EN
    peripherals
        .SYSTIMER
        .conf
        .write(|w| unsafe { w.bits(1 << 24 | 1 << 30) });

    peripherals
        .SYSTIMER
        .int_clr
        .write(|w| unsafe { w.bits(1 << 0) });

    // TARGET0 INT ENA
    peripherals
        .SYSTIMER
        .int_ena
        .write(|w| unsafe { w.bits(1 << 0) });

    peripherals
        .INTERRUPT_CORE0
        .systimer_target0_int_map
        .write(|w| unsafe { w.bits(10) });
    peripherals
        .INTERRUPT_CORE0
        .cpu_int_pri_10
        .write(|w| unsafe { w.bits(1) }); // PRIO = 1
    peripherals
        .INTERRUPT_CORE0
        .cpu_int_enable
        .write(|w| unsafe { w.bits(1 << 10) }); // ENABLE INT 10
}

pub fn init_tasks() {
    task_create(worker_task1);
    task_create(worker_task2);
}

pub extern "C" fn worker_task1() {
    loop {
        compat::work_queue::do_work();
    }
}

pub extern "C" fn worker_task2() {
    loop {
        let mut to_run: SimpleQueue<
            (
                fn(*mut crate::binary::c_types::c_void),
                *mut crate::binary::c_types::c_void,
            ),
            10,
        > = SimpleQueue::new();

        critical_section::with(|_| unsafe {
            for i in 0..TIMERS.len() {
                TIMERS[i] = match &TIMERS[i] {
                    Some(old) => {
                        if old.active && get_systimer_count() >= old.expire {
                            trace!("timer is due....");
                            let fnctn: fn(*mut crate::binary::c_types::c_void) =
                                core::mem::transmute(old.timer_ptr);
                            to_run.enqueue((fnctn, old.arg_ptr));

                            Some(Timer {
                                expire: if old.period != 0 {
                                    get_systimer_count() + old.period
                                } else {
                                    0
                                },
                                active: if old.period != 0 { true } else { false },
                                ..*old
                            })
                        } else {
                            Some(*old)
                        }
                    }
                    None => None,
                };
            }
        });

        // run the due timer callbacks NOT in an interrupt free context
        loop {
            let run_now = to_run.dequeue();
            if run_now.is_none() {
                break;
            }

            let (fnc, arg) = run_now.unwrap();
            trace!("trigger timer....");
            fnc(arg);
            trace!("timer callback called");
        }

        send_data_if_needed();
    }
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
pub fn interrupt2(_trap_frame: &mut TrapFrame) {
    writeln!(Uart, "interrupt 2").ok();
}

// esp32c3_wl_init sets an interrupt handler - not clear who generates the interrupt etc.
#[no_mangle]
pub fn interrupt11(_trap_frame: &mut TrapFrame) {
    writeln!(Uart, "interrupt 11").ok();
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
