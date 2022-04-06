use crate::preempt::preempt::task_switch;
use crate::trace;
use atomic_polyfill::AtomicU64;
use atomic_polyfill::Ordering;
use core::cell::RefCell;
use esp32_hal::pac::{self, Peripherals, TIMG1};
use esp32_hal::prelude::_embedded_hal_timer_CountDown;
use esp32_hal::Timer;
use esp32_hal::{interrupt, Cpu};
use xtensa_lx::mutex::{Mutex, SpinLockMutex};
use xtensa_lx_rt::exception::Context;

// this CAN'T BE correct but this is how it works - figure this out
pub const TICKS_PER_SECOND: u64 = 4_000_000;

const TIMER_DELAY: u64 = 40_000u64;

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

pub fn init_intr11(_peripherals: &Peripherals) {
    // probably even not needed on ESP32C3
    todo!("init_intr11")
}

pub fn setup_timer_isr(timg1: TIMG1) {
    let mut timer1 = Timer::new(timg1);

    interrupt::enable(
        Cpu::ProCpu,
        pac::Interrupt::TG1_T0_LEVEL,
        interrupt::CpuInterrupt::Interrupt20LevelPriority2,
    );
    timer1.start(TIMER_DELAY);
    timer1.listen();

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
        interrupt::clear(
            Cpu::ProCpu,
            interrupt::CpuInterrupt::Interrupt0LevelPriority1,
        );

        let (fnc, arg) = crate::wifi::os_adapter::ISR_INTERRUPT_1;

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

    unsafe {
        TIME.store(
            TIME.load(Ordering::Relaxed) + read_timer_value(),
            Ordering::Relaxed,
        );
    }

    task_switch(context);

    unsafe {
        (&TIMER1).lock(|data| {
            let mut timer1 = data.borrow_mut();
            let timer1 = timer1.as_mut().unwrap();
            timer1.clear_interrupt();
            timer1.start(TIMER_DELAY);
        });
    }
}
