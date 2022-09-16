use super::phy_init_data::PHY_INIT_DATA_DEFAULT;
use crate::binary::include::*;
use esp32s2_hal::prelude::ram;
use log::trace;

pub(crate) unsafe extern "C" fn phy_enable() {
    // quite some code needed here
    trace!("phy_enable - not fully implemented");

    static mut G_IS_PHY_CALIBRATED: bool = false;

    let mut cal_data: [u8; core::mem::size_of::<esp_phy_calibration_data_t>()] =
        [0u8; core::mem::size_of::<esp_phy_calibration_data_t>()];

    critical_section::with(|_| {
        phy_enable_clock();
        //        phy_set_wifi_mode_only(!crate::wifi::BLE_ENABLED);

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

        #[cfg(feature = "ble")]
        {
            extern "C" {
                fn coex_pti_v2();
            }
            coex_pti_v2();
        }
    });
}

pub(crate) unsafe fn phy_enable_clock() {
    trace!("phy_enable_clock");
    const SYSTEM_WIFI_CLK_EN_REG: u32 = 0x3f426000 + 0x090;
    critical_section::with(|_| {
        (SYSTEM_WIFI_CLK_EN_REG as *mut u32).write_volatile(u32::MAX);
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
    regval[0] = ((0x3f41A000 + 0x44) as *const u32).read_volatile();
    regval[1] = ((0x3f41A000 + 0x48) as *const u32).read_volatile();

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

pub(crate) fn init_clocks() {
    log::warn!("init clocks");

    const RTC_CNTL_DIG_PWC_REG: u32 = DR_REG_RTCCNTL_BASE + 0x008C;
    const DR_REG_RTCCNTL_BASE: u32 = 0x3f408000;
    const RTC_CNTL_DIG_ISO_REG: u32 = DR_REG_RTCCNTL_BASE + 0x0090;

    const RTC_CNTL_WIFI_FORCE_PD: u32 = 1 << 17;
    const RTC_CNTL_WIFI_FORCE_ISO: u32 = 1 << 28;
    unsafe {
        (RTC_CNTL_DIG_PWC_REG as *mut u32).write_volatile(
            (RTC_CNTL_DIG_PWC_REG as *mut u32).read_volatile() & !RTC_CNTL_WIFI_FORCE_PD,
        );

        (RTC_CNTL_DIG_ISO_REG as *mut u32).write_volatile(
            (RTC_CNTL_DIG_ISO_REG as *mut u32).read_volatile() & !RTC_CNTL_WIFI_FORCE_ISO,
        );
    }

    // unsafe {
    //     // PERIP_CLK_EN0
    //     ((0x3f426000 + 0x8c) as *mut u32).write_volatile(0xffffffff);
    //     // PERIP_CLK_EN1
    //     ((0x3f426000 + 0x90) as *mut u32).write_volatile(0xffffffff);
    // }

    // // APB_CTRL_WIFI_CLK_EN_REG
    // unsafe {
    //     ((0x3f426000 + 0x90) as *mut u32).write_volatile(0xffffffff);
    // }
}

/****************************************************************************
 * Name: phy_enter_critical
 *
 * Description:
 *   Enter critical state
 *
 * Input Parameters:
 *   None
 *
 * Returned Value:
 *   CPU PS value
 *
 ****************************************************************************/
#[ram]
#[no_mangle]
unsafe extern "C" fn phy_enter_critical() -> u32 {
    trace!("phy_enter_critical");

    core::mem::transmute(critical_section::acquire())
}

/****************************************************************************
 * Name: phy_exit_critical
 *
 * Description:
 *   Exit from critical state
 *
 * Input Parameters:
 *   level - CPU PS value
 *
 * Returned Value:
 *   None
 *
 ****************************************************************************/
#[ram]
#[no_mangle]
unsafe extern "C" fn phy_exit_critical(level: u32) {
    trace!("phy_exit_critical {}", level);

    critical_section::release(core::mem::transmute(level));
}

#[no_mangle]
unsafe extern "C" fn abort() {
    trace!("misc_nvs_deinit")
}

#[no_mangle]
unsafe extern "C" fn misc_nvs_deinit() {
    trace!("misc_nvs_deinit")
}

#[no_mangle]
unsafe extern "C" fn misc_nvs_init() -> i32 {
    trace!("misc_nvs_init");
    0
}

#[no_mangle]
unsafe extern "C" fn misc_nvs_restore() -> i32 {
    todo!("misc_nvs_restore")
}

#[no_mangle]
static mut g_log_mod: i32 = 0;

#[no_mangle]
static mut g_log_level: i32 = 0;

#[no_mangle]
pub static mut g_misc_nvs: &u32 = unsafe { &NVS };

pub static mut NVS: u32 = 0;
