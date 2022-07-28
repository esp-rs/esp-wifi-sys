use crate::binary::include::*;
use crate::compat::common::StrBuf;
use crate::wifi::phy_init_data_esp32c3::PHY_INIT_DATA_DEFAULT;
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

pub(crate) fn init_clocks() {
    unsafe {
        // PERIP_CLK_EN0
        ((0x600c0000 + 0x10) as *mut u32).write_volatile(0xffffffff);
        // PERIP_CLK_EN1
        ((0x600c0000 + 0x14) as *mut u32).write_volatile(0xffffffff);
    }

    // APB_CTRL_WIFI_CLK_EN_REG
    unsafe {
        ((0x60026000 + 0x14) as *mut u32).write_volatile(0xffffffff);
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

pub(crate) unsafe extern "C" fn phy_enable() {
    // quite some code needed here
    trace!("phy_enable - not fully implemented");

    static mut G_IS_PHY_CALIBRATED: bool = false;

    let mut cal_data: [u8; core::mem::size_of::<esp_phy_calibration_data_t>()] =
        [0u8; core::mem::size_of::<esp_phy_calibration_data_t>()];

    let phy_version = get_phy_version_str();
    trace!("phy_version {}", StrBuf::from(phy_version).as_str_ref());

    critical_section::with(|_| {
        phy_enable_clock();
        phy_set_wifi_mode_only(!crate::wifi::BLE_ENABLED);

        if G_IS_PHY_CALIBRATED == false {
            let init_data = &PHY_INIT_DATA_DEFAULT;

            register_chipv7_phy(
                init_data,
                &mut cal_data as *mut _ as *mut crate::binary::include::esp_phy_calibration_data_t,
                esp_phy_calibration_mode_t_PHY_RF_CAL_FULL,
            );

            G_IS_PHY_CALIBRATED = true;
        } else {
            trace!("implement phy_digital_regs_load");
            phy_wakeup_init();
            //phy_digital_regs_load();
            /*
            static inline void phy_digital_regs_load(void)
            {
            if (g_phy_digital_regs_mem != NULL)
                {
                phy_dig_reg_backup(false, g_phy_digital_regs_mem);
                }
            }
            */
        }

        extern "C" {
            fn coex_pti_v2();
        }

        if crate::wifi::BLE_ENABLED {
            coex_pti_v2();
        }
    });
}

pub(crate) unsafe fn phy_enable_clock() {
    //modifyreg32(SYSTEM_WIFI_CLK_EN_REG, 0, SYSTEM_WIFI_CLK_WIFI_BT_COMMON_M);
    trace!("phy_enable_clock");
    // const SYSTEM_WIFI_CLK_EN_REG: u32 = 0x60026000 + 0x014;
    critical_section::with(|_| {
        // something wrong here? with this things will block in phy_enable!!!!! find out what's going on here???
        //    (SYSTEM_WIFI_CLK_EN_REG as *mut u32).write_volatile(0x78078F);
    });

    trace!("phy_enable_clock done!");
}

pub(crate) unsafe extern "C" fn read_mac(
    mac: *mut u8,
    type_: u32,
) -> crate::binary::c_types::c_int {
    trace!("read_mac {:p} {}", mac, type_);

    let mut regval = [0u32; 2];
    let data = &regval as *const _ as *const u8;
    regval[0] = ((0x60008800 + 0x44) as *const u32).read_volatile();
    regval[1] = ((0x60008800 + 0x48) as *const u32).read_volatile();

    for i in 0..6 {
        mac.offset(i)
            .write_volatile(data.offset(5 - i).read_volatile());
    }

    /* ESP_MAC_WIFI_SOFTAP */
    if type_ == 1 {
        let tmp = mac.offset(0).read_volatile();
        for i in 0..64 {
            mac.offset(0).write_volatile(tmp | 0x02);
            mac.offset(0)
                .write_volatile(mac.offset(0).read_volatile() ^ (i << 2));

            if mac.offset(0).read_volatile() != tmp {
                break;
            }
        }
    }

    // ESP_MAC_BT
    if type_ == 2 {
        let tmp = mac.offset(0).read_volatile();
        for i in 0..64 {
            mac.offset(0).write_volatile(tmp | 0x02);
            mac.offset(0)
                .write_volatile(mac.offset(0).read_volatile() ^ (i << 2));

            if mac.offset(0).read_volatile() != tmp {
                break;
            }
        }

        mac.offset(5)
            .write_volatile(mac.offset(5).read_volatile() + 1);
    }

    0
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
