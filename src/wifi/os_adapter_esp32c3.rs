use esp32c3_hal as hal;
use log::trace;

pub(crate) fn chip_ints_on(mask: u32) {
    let cpuint = match mask {
        2 => 1,
        _ => panic!("ints_on mask {} not handled", mask),
    };

    trace!("ints_on n={}", cpuint);

    unsafe {
        (*hal::pac::INTERRUPT_CORE0::PTR)
            .cpu_int_enable
            .modify(|r, w| w.bits(r.bits() | 1 << cpuint));
    }
}

pub(crate) unsafe extern "C" fn wifi_int_disable(
    wifi_int_mux: *mut crate::binary::c_types::c_void,
) -> u32 {
    let res = if riscv::register::mstatus::read().mie() {
        1
    } else {
        0
    };
    riscv::interrupt::disable();

    trace!(
        "wifi_int_disable wifi_int_mux {:p} - return {}",
        wifi_int_mux,
        res,
    );

    res
}

pub(crate) unsafe extern "C" fn wifi_int_restore(
    wifi_int_mux: *mut crate::binary::c_types::c_void,
    tmp: u32,
) {
    trace!(
        "wifi_int_restore wifi_int_mux {:p} tmp {}",
        wifi_int_mux,
        tmp
    );

    if tmp == 1 {
        riscv::interrupt::enable();
    }
}

pub(crate) unsafe extern "C" fn set_intr(
    cpu_no: i32,
    intr_source: u32,
    intr_num: u32,
    intr_prio: i32,
) {
    trace!(
        "set_intr {} {} {} {}",
        cpu_no,
        intr_source,
        intr_num,
        intr_prio
    );

    // this gets called with
    // INFO - set_intr 0 2 1 1 (WIFI_PWR)
    // INFO - set_intr 0 0 1 1 (WIFI_MAC)

    // we do nothing here anymore since all the interrupts are already
    // configured in `setup_timer_isr` and messing with the interrupts will
    // get us into trouble

    // esp32c3_bind_irq(intr_num, intr_source, intr_prio, ESP32C3_INT_LEVEL);

    /* Disable the CPU interrupt. */
    // resetbits(1 << cpuint, INTERRUPT_CPU_INT_ENABLE_REG);

    /* Set the interrupt priority. */
    //  ((0x600c2000 + 0x114 + intr_num * 4) as *mut u32).write_volatile(intr_prio as u32);

    /* Set the interrupt type (Edge or Level). */
    // ----

    /* Map the CPU interrupt ID to the peripheral. */
    //  ((0x600c2000 + intr_source * 4) as *mut u32).write_volatile(intr_num as u32);
}

pub(crate) unsafe extern "C" fn wifi_clock_enable() {
    // modifyreg32(SYSTEM_WIFI_CLK_EN_REG, 0, SYSTEM_WIFI_CLK_WIFI_EN_M);
    trace!("wifi_clock_enable");
    // const SYSTEM_WIFI_CLK_EN_REG: u32 = 0x60026000 + 0x014;
    critical_section::with(|_| {
        //    (SYSTEM_WIFI_CLK_EN_REG as *mut u32).write_volatile(0);
    });
}

pub(crate) unsafe extern "C" fn wifi_clock_disable() {
    trace!("wifi_clock_disable")
}
