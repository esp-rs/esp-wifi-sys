#![allow(unused_variables)]
#![allow(dead_code)]
#![allow(non_snake_case)]
use esp32_hal::macros::ram;

use crate::binary::include::*;
use crate::wifi::phy_init_data_esp32::PHY_INIT_DATA_DEFAULT;
use log::trace;

const DR_REG_DPORT_BASE: u32 = 0x3ff00000;
const DPORT_WIFI_CLK_EN_REG: u32 = DR_REG_DPORT_BASE + 0x0CC;
const DPORT_WIFI_CLK_WIFI_EN: u32 = 0x00000406;
const DPORT_WIFI_CLK_WIFI_EN_V: u32 = 0x406;
const DPORT_WIFI_CLK_WIFI_EN_S: u32 = 0;
const DPORT_WIFI_CLK_WIFI_EN_M: u32 = (DPORT_WIFI_CLK_WIFI_EN_V) << (DPORT_WIFI_CLK_WIFI_EN_S);

// Mask for clock bits used by both WIFI and Bluetooth
const DPORT_WIFI_CLK_WIFI_BT_COMMON_M: u32 = 0x000003c9;

pub(crate) fn chip_ints_on(mask: u32) {
    trace!("chip_ints_on esp32");
    unsafe {
        xtensa_lx::interrupt::enable_mask(1 << 0);
    }
}

#[ram]
pub(crate) fn init_clocks() {
    unsafe {
        let mut regval = getreg32(0x6001f048); /* DR_REG_BB_BASE+48 */
        regval &= !(1 << 14);
        putreg32(regval, 0x6001f048);
    }
}

#[inline(always)]
unsafe fn putreg32(v: u32, r: u32) {
    (r as *mut u32).write_volatile(v);
}

#[inline(always)]
unsafe fn getreg32(r: u32) -> u32 {
    (r as *mut u32).read_volatile()
}

pub(crate) unsafe extern "C" fn wifi_int_disable(
    wifi_int_mux: *mut crate::binary::c_types::c_void,
) -> u32 {
    trace!("wifi_int_disable() esp32");
    core::mem::transmute(critical_section::acquire())
}

pub(crate) unsafe extern "C" fn wifi_int_restore(
    wifi_int_mux: *mut crate::binary::c_types::c_void,
    tmp: u32,
) {
    trace!("wifi_int_restore() esp32");
    critical_section::release(core::mem::transmute(tmp))
}

pub(crate) unsafe extern "C" fn phy_common_clock_disable() {
    trace!("phy_common_clock_disable - not implemented");
}

pub(crate) unsafe extern "C" fn phy_common_clock_enable() {
    trace!("phy_common_clock_enable - not implemented");
}

pub(crate) unsafe extern "C" fn set_intr(
    cpu_no: i32,
    intr_source: u32,
    intr_num: u32,
    intr_prio: i32,
) {
    extern "C" {
        fn intr_matrix_set(cpu_no: u32, model_num: u32, intr_num: u32);
    }
    // Force to bind WiFi interrupt to CPU0
    intr_matrix_set(0, intr_source, intr_num);
}

pub(crate) unsafe extern "C" fn phy_enable() {
    trace!("phy_enable - not fully implemented");

    static mut G_IS_PHY_CALIBRATED: bool = false;

    let mut cal_data: [u8; core::mem::size_of::<esp_phy_calibration_data_t>()] =
        [0u8; core::mem::size_of::<esp_phy_calibration_data_t>()];

    // `get_phy_version_str` will break things - so keep it commented out
    //let phy_version = get_phy_version_str();
    //trace!("phy_version {}", StrBuf::from(phy_version).as_str_ref());

    critical_section::with(|_| {
        let g_phy_rf_en_ts = 0; // TODO
                                // Update WiFi MAC time before WiFi/BT common clock is enabled */
        phy_update_wifi_mac_time(false, g_phy_rf_en_ts);

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
    });
}

/****************************************************************************
 * Name: phy_update_wifi_mac_time
 *
 * Description:
 *   Update WiFi mac timer.
 *
 * Input Parameters:
 *   en_clock_stopped - Check if clock is stopped
 *   now              - time now
 *
 * Returned Value:
 *   NOne
 *
 ****************************************************************************/
pub(crate) unsafe fn phy_update_wifi_mac_time(en_clock_stopped: bool, now: i64) {
    trace!("phy_update_wifi_mac_time");

    static mut G_COMMON_CLOCK_DISABLE_TIME: u32 = 0;

    let diff: u32;

    if en_clock_stopped {
        G_COMMON_CLOCK_DISABLE_TIME = now as u32;
    } else {
        if G_COMMON_CLOCK_DISABLE_TIME != 0 {
            diff = now as u32 - G_COMMON_CLOCK_DISABLE_TIME;
            esp_wifi_internal_update_mac_time(diff);
            G_COMMON_CLOCK_DISABLE_TIME = 0;
        }
    }
}

pub(crate) unsafe fn phy_enable_clock() {
    trace!("phy_enable_clock");

    static mut ENABLE_CNT: u32 = 0;

    if ENABLE_CNT == 0 {
        let ptr = DPORT_WIFI_CLK_EN_REG as *mut u32;
        let old = ptr.read_volatile();
        ptr.write_volatile(old | DPORT_WIFI_CLK_WIFI_BT_COMMON_M);

        ENABLE_CNT += 1;
    }
}

pub(crate) unsafe extern "C" fn read_mac(
    mac: *mut u8,
    type_: u32,
) -> crate::binary::c_types::c_int {
    trace!("read_mac {:p} {}", mac, type_);

    let mut regval = [0u32; 2];
    let data = &regval as *const _ as *const u8;
    regval[0] = (0x3ff5a004 as *const u32).read_volatile();
    regval[1] = (0x3ff5a008 as *const u32).read_volatile();

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
    trace!("wifi_clock_enable");

    let ptr = DPORT_WIFI_CLK_EN_REG as *mut u32;
    let old = ptr.read_volatile();
    ptr.write_volatile(old | DPORT_WIFI_CLK_WIFI_EN_M);
}

pub(crate) unsafe extern "C" fn wifi_clock_disable() {
    trace!("wifi_clock_disable");

    let ptr = DPORT_WIFI_CLK_EN_REG as *mut u32;
    let old = ptr.read_volatile();
    ptr.write_volatile(old & !DPORT_WIFI_CLK_WIFI_EN_M);
}
