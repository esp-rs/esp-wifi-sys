use super::phy_init_data::PHY_INIT_DATA_DEFAULT;
use crate::binary::include::*;
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

        #[cfg(feature = "phy_enable_usb")]
        {
            extern "C" {
                pub fn phy_bbpll_en_usb(param: bool);
            }

            phy_bbpll_en_usb(true);
        }

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
    const SYSTEM_WIFI_CLK_EN_REG: u32 = 0x60026000 + 0x014;
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
    regval[0] = ((0x60007000 + 0x44) as *const u32).read_volatile();
    regval[1] = ((0x60007000 + 0x48) as *const u32).read_volatile();

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
pub static mut g_misc_nvs: u32 = 0;
