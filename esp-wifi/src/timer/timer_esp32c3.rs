use crate::{
    binary,
    hal::{
        interrupt::{self, TrapFrame},
        peripherals::{self, Interrupt},
        prelude::*,
        riscv,
        systimer::{Alarm, Periodic, SystemTimer, Target},
    },
};

pub use super::arch_specific::{get_systimer_count, yield_task, TICKS_PER_SECOND};
use super::arch_specific::{setup_multitasking, setup_timer};

pub fn setup_timer_isr(systimer: Alarm<Target, 0>) {
    setup_timer(systimer);

    #[cfg(feature = "wifi")]
    {
        unwrap!(interrupt::enable(
            Interrupt::WIFI_MAC,
            interrupt::Priority::Priority1
        ));
        unwrap!(interrupt::enable(
            Interrupt::WIFI_PWR,
            interrupt::Priority::Priority1
        ));
    }

    #[cfg(feature = "ble")]
    {
        unwrap!(interrupt::enable(
            Interrupt::RWBT,
            interrupt::Priority::Priority1
        ));
        unwrap!(interrupt::enable(
            Interrupt::RWBLE,
            interrupt::Priority::Priority1
        ));
        unwrap!(interrupt::enable(
            Interrupt::BT_BB,
            interrupt::Priority::Priority1
        ));
    }

    setup_multitasking();
}

#[cfg(feature = "wifi")]
#[interrupt]
fn WIFI_MAC() {
    unsafe {
        let (fnc, arg) = crate::wifi::os_adapter::ISR_INTERRUPT_1;

        trace!("interrupt WIFI_MAC {:?} {:?}", fnc, arg);

        if !fnc.is_null() {
            let fnc: fn(*mut binary::c_types::c_void) = core::mem::transmute(fnc);
            fnc(arg);
        }

        trace!("interrupt 1 done");
    };
}

#[cfg(feature = "wifi")]
#[interrupt]
fn WIFI_PWR() {
    unsafe {
        let (fnc, arg) = crate::wifi::os_adapter::ISR_INTERRUPT_1;

        trace!("interrupt WIFI_PWR {:?} {:?}", fnc, arg);

        if !fnc.is_null() {
            let fnc: fn(*mut binary::c_types::c_void) = core::mem::transmute(fnc);
            fnc(arg);
        }

        trace!("interrupt 1 done");
    };
}

#[cfg(feature = "ble")]
#[interrupt]
fn RWBT() {
    unsafe {
        let (fnc, arg) = crate::ble::btdm::ble_os_adapter_chip_specific::BT_INTERRUPT_FUNCTION5;

        trace!("interrupt RWBT {:?} {:?}", fnc, arg);

        if !fnc.is_null() {
            let fnc: fn(*mut binary::c_types::c_void) = core::mem::transmute(fnc);
            fnc(arg);
        }

        trace!("interrupt 5 done");
    };
}

#[cfg(feature = "ble")]
#[interrupt]
fn RWBLE() {
    unsafe {
        let (fnc, arg) = crate::ble::btdm::ble_os_adapter_chip_specific::BT_INTERRUPT_FUNCTION5;

        trace!("interrupt RWBLE {:?} {:?}", fnc, arg);

        if !fnc.is_null() {
            let fnc: fn(*mut binary::c_types::c_void) = core::mem::transmute(fnc);
            fnc(arg);
        }

        trace!("interrupt 5 done");
    };
}

#[cfg(feature = "ble")]
#[interrupt]
fn BT_BB(_trap_frame: &mut TrapFrame) {
    unsafe {
        let (fnc, arg) = crate::ble::btdm::ble_os_adapter_chip_specific::BT_INTERRUPT_FUNCTION8;

        trace!("interrupt BT_BB {:?} {:?}", fnc, arg);

        if !fnc.is_null() {
            let fnc: fn(*mut binary::c_types::c_void) = core::mem::transmute(fnc);
            fnc(arg);
        }

        trace!("interrupt 8 done");
    };
}
