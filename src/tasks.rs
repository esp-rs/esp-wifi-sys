use esp_alloc::memory_fence;
use log::{debug, trace};

use crate::{
    compat::{
        self,
        queue::SimpleQueue,
        timer_compat::{Timer, TIMERS},
    },
    preempt::preempt::task_create,
    timer::get_systimer_count,
    wifi::send_data_if_needed,
};

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
                memory_fence();
                TIMERS[i] = match &TIMERS[i] {
                    Some(old) => {
                        if old.active && get_systimer_count() >= old.expire {
                            debug!("timer is due.... {:p}", old.ptimer);
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
