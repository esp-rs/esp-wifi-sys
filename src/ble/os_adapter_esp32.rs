use crate::binary::{
    c_types,
    include::{
        esp_bt_controller_config_t, esp_bt_mode_t, esp_bt_mode_t_ESP_BT_MODE_BLE,
        esp_bt_mode_t_ESP_BT_MODE_BTDM, esp_bt_mode_t_ESP_BT_MODE_CLASSIC_BT,
        esp_bt_mode_t_ESP_BT_MODE_IDLE,
    },
};

pub static mut ISR_INTERRUPT_5: (
    *mut crate::binary::c_types::c_void,
    *mut crate::binary::c_types::c_void,
) = (core::ptr::null_mut(), core::ptr::null_mut());

pub static mut ISR_INTERRUPT_8: (
    *mut crate::binary::c_types::c_void,
    *mut crate::binary::c_types::c_void,
) = (core::ptr::null_mut(), core::ptr::null_mut());

pub static mut ISR_INTERRUPT_7: (
    *mut crate::binary::c_types::c_void,
    *mut crate::binary::c_types::c_void,
) = (core::ptr::null_mut(), core::ptr::null_mut());

#[repr(C)]
struct btdm_dram_available_region_t {
    mode: esp_bt_mode_t,
    start: u32, // ptr
    end: u32,   // ptr
}

const SOC_MEM_BT_DATA_START: u32 = 0x3ffae6e0;
const SOC_MEM_BT_DATA_END: u32 = 0x3ffaff10;
const SOC_MEM_BT_EM_BTDM0_START: u32 = 0x3ffb0000;
const SOC_MEM_BT_EM_BTDM0_END: u32 = 0x3ffb09a8;
const SOC_MEM_BT_EM_BLE_START: u32 = 0x3ffb09a8;
const SOC_MEM_BT_EM_BLE_END: u32 = 0x3ffb1ddc;
const SOC_MEM_BT_EM_BTDM1_START: u32 = 0x3ffb1ddc;
const SOC_MEM_BT_EM_BTDM1_END: u32 = 0x3ffb2730;
const SOC_MEM_BT_EM_BREDR_START: u32 = 0x3ffb2730;
const SOC_MEM_BT_BSS_START: u32 = 0x3ffb8000;
const SOC_MEM_BT_BSS_END: u32 = 0x3ffb9a20;
const SOC_MEM_BT_MISC_START: u32 = 0x3ffbdb28;
const SOC_MEM_BT_MISC_END: u32 = 0x3ffbdb5c;

const SOC_MEM_BT_EM_BREDR_REAL_END: u32 = 0x3ffb6388; //  (SOC_MEM_BT_EM_BREDR_NO_SYNC_END + CONFIG_BTDM_CTRL_BR_EDR_MAX_SYNC_CONN_EFF * SOC_MEM_BT_EM_PER_SYNC_SIZE);

static BTDM_DRAM_AVAILABLE_REGION: [btdm_dram_available_region_t; 7] = [
    /* following is .data */
    btdm_dram_available_region_t {
        mode: esp_bt_mode_t_ESP_BT_MODE_BTDM,
        start: SOC_MEM_BT_DATA_START,
        end: SOC_MEM_BT_DATA_END,
    },
    /* following is memory which HW will use */
    btdm_dram_available_region_t {
        mode: esp_bt_mode_t_ESP_BT_MODE_BTDM,
        start: SOC_MEM_BT_EM_BTDM0_START,
        end: SOC_MEM_BT_EM_BTDM0_END,
    },
    btdm_dram_available_region_t {
        mode: esp_bt_mode_t_ESP_BT_MODE_BLE,
        start: SOC_MEM_BT_EM_BLE_START,
        end: SOC_MEM_BT_EM_BLE_END,
    },
    btdm_dram_available_region_t {
        mode: esp_bt_mode_t_ESP_BT_MODE_BTDM,
        start: SOC_MEM_BT_EM_BTDM1_START,
        end: SOC_MEM_BT_EM_BTDM1_END,
    },
    btdm_dram_available_region_t {
        mode: esp_bt_mode_t_ESP_BT_MODE_CLASSIC_BT,
        start: SOC_MEM_BT_EM_BREDR_START,
        end: SOC_MEM_BT_EM_BREDR_REAL_END,
    },
    /* following is .bss */
    btdm_dram_available_region_t {
        mode: esp_bt_mode_t_ESP_BT_MODE_BTDM,
        start: SOC_MEM_BT_BSS_START,
        end: SOC_MEM_BT_BSS_END,
    },
    btdm_dram_available_region_t {
        mode: esp_bt_mode_t_ESP_BT_MODE_BTDM,
        start: SOC_MEM_BT_MISC_START,
        end: SOC_MEM_BT_MISC_END,
    },
];

pub(crate) fn create_ble_config() -> esp_bt_controller_config_t {
    esp_bt_controller_config_t {
        controller_task_stack_size: 4096,
        controller_task_prio: 110,
        hci_uart_no: 1,
        hci_uart_baudrate: 921600,
        scan_duplicate_mode: 0,
        scan_duplicate_type: 0,
        normal_adv_size: 200,
        mesh_adv_size: 0,
        send_adv_reserved_size: 1000,
        controller_debug_flag: 0 << 0,
        mode: 0x01, // BLE
        ble_max_conn: 3,
        bt_max_acl_conn: 0,
        bt_sco_datapath: 0,
        auto_latency: false,
        bt_legacy_auth_vs_evt: false,
        bt_max_sync_conn: 1,
        ble_sca: 1,
        pcm_role: 0,
        pcm_polar: 0,
        hli: false,
        dup_list_refresh_period: 0,
        magic: 0x20221207,
    }
}

pub(crate) fn btdm_controller_mem_init() {
    extern "C" {
        static _data_start_btdm: u32;
        static _data_start_btdm_rom: u32;
        static _data_end_btdm: u32;
    }

    // initialise .data section
    unsafe {
        let len = (&_data_end_btdm as *const _ as *const u8 as usize)
            - (&_data_start_btdm as *const _ as *const u8 as usize);

        core::ptr::copy_nonoverlapping(
            _data_start_btdm_rom as *const u8,
            &_data_start_btdm as *const _ as *mut u8,
            len,
        );

        log::debug!(
            "btdm_controller_mem_init {:p} {:p} {}",
            _data_start_btdm_rom as *const u8,
            &_data_start_btdm as *const _ as *mut u8,
            len,
        );
    }

    // initialize em, .bss section
    let btdm_dram_regions = BTDM_DRAM_AVAILABLE_REGION.len();

    for i in 1..btdm_dram_regions {
        if BTDM_DRAM_AVAILABLE_REGION[i].mode != esp_bt_mode_t_ESP_BT_MODE_IDLE {
            unsafe {
                core::ptr::write_bytes(
                    BTDM_DRAM_AVAILABLE_REGION[i].start as *mut u8,
                    0x0,
                    (BTDM_DRAM_AVAILABLE_REGION[i].end - BTDM_DRAM_AVAILABLE_REGION[i].start)
                        as usize,
                );
            }
            log::debug!(
                ".bss initialise {:x} - {:x}\n",
                BTDM_DRAM_AVAILABLE_REGION[i].start,
                BTDM_DRAM_AVAILABLE_REGION[i].end
            );
        }
    }

    log::debug!("btdm_controller_mem_init done");
}

pub(crate) fn bt_periph_module_enable() {
    unsafe {
        const DR_REG_DPORT_BASE: u32 = 0x3ff00000;
        const DPORT_WIFI_CLK_EN_REG: u32 = DR_REG_DPORT_BASE + 0x0CC;
        const DPORT_CORE_RST_EN_REG: u32 = DR_REG_DPORT_BASE + 0x0D0;
        const DPORT_WIFI_CLK_BT_EN: u32 = 0x30800;

        let ptr = DPORT_WIFI_CLK_EN_REG as *mut u32;
        let old = ptr.read_volatile();
        ptr.write_volatile(old | DPORT_WIFI_CLK_BT_EN);

        let ptr = DPORT_CORE_RST_EN_REG as *mut u32;
        let old = ptr.read_volatile();
        ptr.write_volatile(old | 0);
    }
    // bt_periph_module_enable(PERIPH_BT_MODULE);
    // modifyreg32(get_clk_en_reg(periph), 0, get_clk_en_mask(periph));
    // modifyreg32(get_rst_en_reg(periph), get_rst_en_mask(periph, true), 0);
}

pub(crate) fn disable_sleep_mode() {
    extern "C" {
        fn btdm_controller_set_sleep_mode(mode: u8);
    }

    const BTDM_MODEM_SLEEP_MODE_NONE: u8 = 0;

    unsafe {
        btdm_controller_set_sleep_mode(BTDM_MODEM_SLEEP_MODE_NONE);
    }
}

pub(crate) unsafe extern "C" fn coex_bt_wakeup_request() -> bool {
    log::debug!("coex_bt_wakeup_request");
    #[cfg(coex)]
    return async_wakeup_request(BTDM_ASYNC_WAKEUP_REQ_COEX);

    #[cfg(not(coex))]
    true
}

pub(crate) unsafe extern "C" fn coex_bt_wakeup_request_end() {
    log::warn!("coex_bt_wakeup_request_end");

    #[cfg(coex)]
    async_wakeup_request_end(BTDM_ASYNC_WAKEUP_REQ_COEX);
}

#[allow(unused_variables)]
pub(crate) unsafe extern "C" fn coex_bt_request(event: u32, latency: u32, duration: u32) -> i32 {
    log::debug!("coex_bt_request");
    extern "C" {
        #[cfg(coex)]
        fn coex_bt_request(event: u32, latency: u32, duration: u32) -> i32;
    }

    #[cfg(coex)]
    return coex_bt_request(event, latency, duration);

    #[cfg(not(coex))]
    0
}

#[allow(unused_variables)]
pub(crate) unsafe extern "C" fn coex_bt_release(event: u32) -> i32 {
    log::debug!("coex_bt_release");
    extern "C" {
        #[cfg(coex)]
        fn coex_bt_release(event: u32) -> i32;
    }

    #[cfg(coex)]
    return coex_bt_release(event);

    #[cfg(not(coex))]
    0
}

pub(crate) unsafe extern "C" fn coex_register_bt_cb_wrapper(
    callback: unsafe extern "C" fn(),
) -> i32 {
    log::warn!("coex_register_bt_cb {:p}", callback);
    extern "C" {
        #[cfg(coex)]
        fn coex_register_bt_cb(callback: unsafe extern "C" fn()) -> i32;
    }

    #[cfg(coex)]
    return coex_register_bt_cb(callback);

    #[cfg(not(coex))]
    0
}

pub(crate) unsafe extern "C" fn coex_bb_reset_lock() -> u32 {
    log::debug!("coex_bb_reset_lock");
    extern "C" {
        #[cfg(coex)]
        fn coex_bb_reset_lock() -> u32;
    }

    #[cfg(coex)]
    return coex_bb_reset_lock();

    #[cfg(not(coex))]
    0
}

#[allow(unused_variables)]
pub(crate) unsafe extern "C" fn coex_bb_reset_unlock(event: u32) {
    log::debug!("coex_bb_reset_unlock");
    extern "C" {
        #[cfg(coex)]
        fn coex_bb_reset_unlock(event: u32);
    }

    #[cfg(coex)]
    coex_bb_reset_unlock(event);
}

pub(crate) unsafe extern "C" fn coex_schm_register_btdm_callback_wrapper(
    callback: unsafe extern "C" fn(),
) -> i32 {
    log::warn!("coex_schm_register_btdm_callback {:p}", callback);
    extern "C" {
        #[cfg(coex)]
        fn coex_schm_register_btdm_callback(callback: unsafe extern "C" fn()) -> i32;
    }

    #[cfg(coex)]
    return coex_schm_register_btdm_callback(callback);

    #[cfg(not(coex))]
    0
}

pub(crate) unsafe extern "C" fn coex_schm_interval_get() -> u32 {
    log::debug!("coex_schm_interval_get");

    #[cfg(coex)]
    return crate::binary::include::coex_schm_interval_get();

    #[cfg(not(coex))]
    0
}

pub(crate) unsafe extern "C" fn coex_schm_curr_period_get() -> u8 {
    log::debug!("coex_schm_curr_period_get");
    // BEWARE: One might expect to call coex_schm_curr_period_get
    //crate::binary::include::coex_schm_curr_period_get()

    #[cfg(coex)]
    return crate::binary::include::coex_schm_interval_get() as u8;

    #[cfg(not(coex))]
    0
}

pub(crate) unsafe extern "C" fn coex_schm_curr_phase_get() -> *const () {
    log::debug!("coex_schm_curr_phase_get");

    #[cfg(coex)]
    return crate::binary::include::coex_schm_curr_phase_get() as *const ();

    #[cfg(not(coex))]
    return 0 as *const ();
}

#[allow(unused_variables)]
pub(crate) unsafe extern "C" fn coex_wifi_channel_get(primary: *mut u8, secondary: *mut u8) -> i32 {
    log::warn!("coex_wifi_channel_get");
    extern "C" {
        #[cfg(coex)]
        fn coex_wifi_channel_get(primary: *mut u8, secondary: *mut u8) -> i32;
    }

    #[cfg(coex)]
    return coex_wifi_channel_get(primary, secondary);

    #[cfg(not(coex))]
    -1
}

#[allow(unused_variables)]
pub(crate) unsafe extern "C" fn coex_register_wifi_channel_change_callback(
    callback: unsafe extern "C" fn(),
) -> i32 {
    log::warn!("coex_register_wifi_channel_change_callback");
    extern "C" {
        #[cfg(coex)]
        fn coex_register_wifi_channel_change_callback(callback: unsafe extern "C" fn()) -> i32;
    }

    #[cfg(coex)]
    return coex_register_wifi_channel_change_callback(callback);

    #[cfg(not(coex))]
    0
}

pub(crate) unsafe extern "C" fn set_isr(n: i32, f: unsafe extern "C" fn(), arg: *const ()) -> i32 {
    log::trace!("set_isr called {} {:p} {:p}", n, f, arg);

    match n {
        5 => {
            ISR_INTERRUPT_5 = (f as *mut c_types::c_void, arg as *mut c_types::c_void);
        }
        7 => {
            ISR_INTERRUPT_7 = (f as *mut c_types::c_void, arg as *mut c_types::c_void);
        }
        8 => {
            ISR_INTERRUPT_8 = (f as *mut c_types::c_void, arg as *mut c_types::c_void);
        }
        _ => panic!("set_isr - unsupported interrupt number {}", n),
    }

    0
}

pub(crate) unsafe extern "C" fn ints_on(mask: u32) {
    log::trace!("chip_ints_on esp32 {:b}", mask);
    xtensa_lx::interrupt::enable_mask(mask);
}

#[cfg(coex)]
const BTDM_ASYNC_WAKEUP_REQ_HCI: i32 = 0;

#[cfg(coex)]
const BTDM_ASYNC_WAKEUP_REQ_COEX: i32 = 1;

//const BTDM_ASYNC_WAKEUP_REQMAX: i32 = 2;

/****************************************************************************
 * Name: async_wakeup_request
 *
 * Description:
 *   Request the BLE Controller to wakeup
 *
 * Input Parameters:
 *   event - the event that triggered the wakeup
 *
 * Returned Value:
 *   true if request lock is needed, false otherwise
 *
 ****************************************************************************/

#[cfg(coex)]
fn async_wakeup_request(event: i32) -> bool {
    let request_lock: bool;
    let mut do_wakeup_request = false;

    match event {
        BTDM_ASYNC_WAKEUP_REQ_HCI => {
            request_lock = true;
        }
        BTDM_ASYNC_WAKEUP_REQ_COEX => {
            request_lock = false;
        }
        _ => {
            return false;
        }
    }

    extern "C" {
        fn btdm_power_state_active() -> bool;
        fn btdm_wakeup_request(request_lock: bool);
    }

    if !unsafe { btdm_power_state_active() } {
        do_wakeup_request = true;
        unsafe { btdm_wakeup_request(request_lock) };
    }

    return do_wakeup_request;
}

/****************************************************************************
 * Name: async_wakeup_request_end
 *
 * Description:
 *   Finish a wakeup request
 *
 * Input Parameters:
 *   event - the event that triggered the wakeup
 *
 * Returned Value:
 *   true if request lock is needed, false otherwise
 *
 ****************************************************************************/

#[cfg(coex)]
fn async_wakeup_request_end(event: i32) {
    let request_lock: bool;

    match event {
        BTDM_ASYNC_WAKEUP_REQ_HCI => {
            request_lock = true;
        }
        BTDM_ASYNC_WAKEUP_REQ_COEX => {
            request_lock = false;
        }
        _ => {
            return;
        }
    }

    extern "C" {
        // this isn't found anywhere ... not a ROM function
        // not in any of the libs - but the code will never call this anyway

        // fn btdm_wakeup_request_end();
    }
    if request_lock {
        // unsafe { btdm_wakeup_request_end() };
    }

    return;
}
