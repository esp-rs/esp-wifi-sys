use crate::{
    binary::include::{esp_timer_create_args_t, esp_timer_handle_t},
    trace,
};

static ESP_FAKE_TIMER: () = ();

const TIMER_INITIALIZED_VAL: u32 = 0x5aa5a55a;

#[derive(Debug, Clone, Copy)]
pub struct Timer {
    pub ptimer: *mut crate::binary::c_types::c_void,
    pub expire: u64,
    pub period: u64,
    pub active: bool,
    pub timer_ptr: *mut crate::binary::c_types::c_void,
    pub arg_ptr: *mut crate::binary::c_types::c_void,
}

pub static mut TIMERS: [Option<Timer>; 20] = [None; 20];

pub fn compat_timer_arm(ptimer: *mut crate::binary::c_types::c_void, tmout: u32, repeat: bool) {
    compat_timer_arm_us(ptimer, tmout * 1000, repeat);
}

pub fn compat_timer_arm_us(ptimer: *mut crate::binary::c_types::c_void, us: u32, repeat: bool) {
    crate::debug!(
        "timer_arm_us, current time {}",
        crate::timer::get_systimer_count()
    );

    let ticks = us as u64 * (crate::timer::TICKS_PER_SECOND / 1_000_000);
    crate::debug!("timer_arm_us {:p} {} {}", ptimer, ticks, repeat);
    critical_section::with(|_| unsafe {
        for i in 0..TIMERS.len() {
            if let Some(mut timer) = TIMERS[i] {
                if timer.ptimer == ptimer {
                    trace!("found timer ...");
                    timer.expire = ticks as u64 + crate::timer::get_systimer_count();
                    timer.active = true;
                    if repeat {
                        timer.period = ticks as u64;
                    }
                    TIMERS[i] = Some(timer);
                    break;
                }
            }
        }
    });
}

pub fn compat_timer_disarm(ptimer: *mut crate::binary::c_types::c_void) {
    crate::debug!("timer_disarm {:p}", ptimer);
    critical_section::with(|_| unsafe {
        for i in 0..TIMERS.len() {
            if let Some(mut timer) = TIMERS[i] {
                if timer.ptimer == ptimer {
                    trace!("found timer ...");
                    timer.active = false;
                    TIMERS[i] = Some(timer);
                    break;
                }
            }
        }
    });
}

pub fn compat_timer_done(ptimer: *mut crate::binary::c_types::c_void) {
    crate::debug!("timer_done {:p}", ptimer);
    critical_section::with(|_| unsafe {
        for i in 0..TIMERS.len() {
            if let Some(timer) = TIMERS[i] {
                if timer.ptimer == ptimer {
                    trace!("found timer ...");
                    TIMERS[i] = None;
                    break;
                }
            }
        }
    });
}

pub fn compat_timer_setfn(
    ptimer: *mut crate::binary::c_types::c_void,
    pfunction: *mut crate::binary::c_types::c_void,
    parg: *mut crate::binary::c_types::c_void,
) {
    let ets_timer = ptimer as *mut crate::binary::include::ets_timer;
    crate::debug!("timer_setfn {:p} {:p} {:p}", ptimer, pfunction, parg,);

    critical_section::with(|_| unsafe {
        if (*ets_timer).expire != TIMER_INITIALIZED_VAL {
            (*ets_timer).priv_ = core::ptr::null_mut();

            let mut success = false;
            for i in 0..TIMERS.len() {
                if TIMERS[i].is_none() {
                    TIMERS[i] = Some(Timer {
                        ptimer: ptimer,
                        expire: 0,
                        period: 0,
                        active: false,
                        timer_ptr: pfunction,
                        arg_ptr: parg,
                    });
                    (*ets_timer).priv_ = i as *mut crate::binary::c_types::c_void;
                    (*ets_timer).expire = TIMER_INITIALIZED_VAL;
                    success = true;
                    break;
                }
            }

            if !success {
                panic!("Ran out of timers!");
            }
        }
    });
}

pub fn compat_esp_timer_create(
    args: *const esp_timer_create_args_t,
    mut out_handle: *mut esp_timer_handle_t,
) -> i32 {
    unsafe {
        crate::debug!(
            "esp_timer_create {:?} {:?} {:p}",
            (*args).callback,
            (*args).arg,
            out_handle
        );
    }

    let args = args as *const esp_timer_create_args_t;

    critical_section::with(|_| unsafe {
        let mut success = false;
        for i in 0..TIMERS.len() {
            crate::debug!("esp_timer_create {}", i);
            if TIMERS[i].is_none() {
                TIMERS[i] = Some(Timer {
                    ptimer: &ESP_FAKE_TIMER as *const _ as *mut crate::binary::c_types::c_void,
                    expire: 0,
                    period: 0,
                    active: false,
                    timer_ptr: core::mem::transmute((*args).callback.unwrap()),
                    arg_ptr: (*args).arg,
                });
                out_handle = &ESP_FAKE_TIMER as *const _ as *mut esp_timer_handle_t;
                success = true;
                crate::debug!("esp_timer_create {:p} {:p}", args, out_handle);

                break;
            }
        }
        if !success {
            panic!("ran out of timers");
        }
    });

    0
}
