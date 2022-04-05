struct XtensaSingleCoreCriticalSection;

critical_section::custom_impl!(XtensaSingleCoreCriticalSection);

static mut LAST_ENABLED_INTERRUPT_MASK: u32 = 0;

/// THIS IS NOT MULTICORE SAFE
unsafe impl critical_section::Impl for XtensaSingleCoreCriticalSection {
    unsafe fn acquire() -> u8 {
        // keep debug
        let mask = xtensa_lx::interrupt::set_mask(
            xtensa_lx_rt::interrupt::CpuInterruptLevel::Level6.mask(),
        );

        if mask & !xtensa_lx_rt::interrupt::CpuInterruptLevel::Level6.mask() != 0 {
            LAST_ENABLED_INTERRUPT_MASK = mask;
            1
        } else {
            0
        }
    }

    unsafe fn release(token: u8) {
        if token != 0 {
            xtensa_lx::interrupt::set_mask(LAST_ENABLED_INTERRUPT_MASK);
        }
    }
}
