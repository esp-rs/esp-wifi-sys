use core::cell::RefCell;

use atomic_polyfill::{AtomicU64, Ordering};
use esp32_hal::{
    clock::Clocks,
    interrupt,
    pac::{self, TIMG1},
    prelude::_embedded_hal_timer_CountDown,
    Cpu, Timer,
};
use log::trace;
use xtensa_lx::mutex::{Mutex, SpinLockMutex};
use xtensa_lx_rt::exception::Context;

use crate::preempt::preempt::task_switch;

pub const TICKS_PER_SECOND: u64 = 40_000_000;

#[cfg(debug_assertions)]
const TIMER_DELAY: fugit::MicrosDurationU64 = fugit::MicrosDurationU64::millis(13);
#[cfg(not(debug_assertions))]
const TIMER_DELAY: fugit::MicrosDurationU64 = fugit::MicrosDurationU64::millis(3);

static mut TIMER1: SpinLockMutex<RefCell<Option<Timer<TIMG1>>>> =
    SpinLockMutex::new(RefCell::new(None));

static mut TIME: AtomicU64 = AtomicU64::new(0);

pub fn get_systimer_count() -> u64 {
    unsafe { TIME.load(Ordering::Relaxed) + read_timer_value() }
}

fn read_timer_value() -> u64 {
    let value = unsafe {
        let timg1 = esp32_hal::pac::TIMG1::ptr();
        (*timg1).t0update.write(|w| w.bits(1));
        (*timg1).t0lo.read().bits() as u64 | (((*timg1).t0hi.read().bits() as u64) << 32u64)
    };
    value
}

pub fn setup_timer_isr(timg1: TIMG1, clocks: &Clocks) {
    let mut timer1 = Timer::new(timg1, clocks.apb_clock);
    interrupt::enable(
        Cpu::ProCpu,
        pac::Interrupt::TG1_T0_LEVEL,
        interrupt::CpuInterrupt::Interrupt20LevelPriority2,
    );
    timer1.listen();
    timer1.start(TIMER_DELAY.convert());
    unsafe {
        (&TIMER1).lock(|data| (*data).replace(Some(timer1)));
    }

    unsafe {
        xtensa_lx::interrupt::disable();
        xtensa_lx::interrupt::enable_mask(
            xtensa_lx_rt::interrupt::CpuInterruptLevel::Level2.mask()
                | xtensa_lx_rt::interrupt::CpuInterruptLevel::Level6.mask(),
        );
    }

    while unsafe {
        crate::preempt::preempt::FIRST_SWITCH.load(core::sync::atomic::Ordering::Relaxed)
    } {}
}

#[no_mangle]
pub fn level1_interrupt(_context: &mut Context) {
    trace!("Interrupt 1");

    unsafe {
        let mut interrupt = 0;

        let mut cpu_int: u32;
        core::arch::asm!("rsr.226  {0}", out(reg) cpu_int, options(nostack)); // 226 = "intset"
                                                                              //let status = interrupt::get_status(Cpu::ProCpu);
        trace!("interrupt 1 status = {:b}", cpu_int);

        if cpu_int & 0b10000000 != 0 {
            interrupt = 7;
            let clear = 1 << 7;
            core::arch::asm!("wsr.227  {0}", in(reg) clear, options(nostack)); // 227 = "intclear"
        } else if cpu_int & 0b100000000 != 0 {
            interrupt = 8;
            let clear = 1 << 8;
            core::arch::asm!("wsr.227  {0}", in(reg) clear, options(nostack)); // 227 = "intclear"

            interrupt::clear(
                Cpu::ProCpu,
                interrupt::CpuInterrupt::Interrupt8LevelPriority1,
            );
        } else if cpu_int & 0b100000 != 0 {
            interrupt = 5;
            let clear = 1 << 5;
            core::arch::asm!("wsr.227  {0}", in(reg) clear, options(nostack)); // 227 = "intclear"

            interrupt::clear(
                Cpu::ProCpu,
                interrupt::CpuInterrupt::Interrupt5LevelPriority1,
            );
        } else {
            interrupt::clear(
                Cpu::ProCpu,
                interrupt::CpuInterrupt::Interrupt0LevelPriority1,
            );
        }
        trace!("interrupt no = {}", interrupt);

        let (fnc, arg) = match interrupt {
            0 => crate::wifi::os_adapter::ISR_INTERRUPT_1,
            5 => crate::ble::ble_os_adapter_chip_specific::ISR_INTERRUPT_5,
            7 => crate::ble::ble_os_adapter_chip_specific::ISR_INTERRUPT_7,
            8 => crate::ble::ble_os_adapter_chip_specific::ISR_INTERRUPT_8,
            _ => panic!("Don't know how to handle interrupt {}", interrupt),
        };

        trace!("interrupt 1 {:p} {:p}", fnc, arg);

        if !fnc.is_null() {
            let fnc: fn(*mut crate::binary::c_types::c_void) = core::mem::transmute(fnc);
            fnc(arg);
        }

        trace!("interrupt 1 done");
    };
}

#[no_mangle]
pub fn level2_interrupt(context: &mut Context) {
    interrupt::clear(
        Cpu::ProCpu,
        interrupt::CpuInterrupt::Interrupt20LevelPriority2,
    );

    task_switch(context);

    unsafe {
        (&TIMER1).lock(|data| {
            crate::memory_fence::memory_fence();

            let mut timer1 = data.borrow_mut();
            let timer1 = timer1.as_mut().unwrap();

            let ticks = timer1.read_raw();
            TIME.store(TIME.load(Ordering::Relaxed) + ticks, Ordering::Relaxed);

            timer1.clear_interrupt();
            timer1.start(TIMER_DELAY.convert());
        });
    }
}
