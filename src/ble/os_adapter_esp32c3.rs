use crate::binary::include::esp_bt_controller_config_t;
use esp32c3_hal as hal;
use log::trace;

pub(crate) static mut BT_INTERRUPT_FUNCTION5: (
    *mut crate::binary::c_types::c_void,
    *mut crate::binary::c_types::c_void,
) = (core::ptr::null_mut(), core::ptr::null_mut());

pub(crate) static mut BT_INTERRUPT_FUNCTION8: (
    *mut crate::binary::c_types::c_void,
    *mut crate::binary::c_types::c_void,
) = (core::ptr::null_mut(), core::ptr::null_mut());

extern "C" {
    fn btdm_controller_rom_data_init() -> i32;
}

pub(crate) fn create_ble_config() -> esp_bt_controller_config_t {
    esp_bt_controller_config_t {
        magic: 0x5A5AA5A5,
        version: 0x02212090,
        controller_task_stack_size: 8192,
        controller_task_prio: 200,
        controller_task_run_cpu: 0,
        bluetooth_mode: 1,
        ble_max_act: 10,
        sleep_mode: 0,
        sleep_clock: 0,
        ble_st_acl_tx_buf_nb: 0,
        ble_hw_cca_check: 0,
        ble_adv_dup_filt_max: 30,
        coex_param_en: false,
        ce_len_type: 0,
        coex_use_hooks: false,
        hci_tl_type: 1,
        hci_tl_funcs: core::ptr::null_mut(),
        txant_dft: 0,
        rxant_dft: 0,
        txpwr_dft: 7,
        cfg_mask: 1,
        scan_duplicate_mode: 0,
        scan_duplicate_type: 0,
        normal_adv_size: 20,
        mesh_adv_size: 0,
        coex_phy_coded_tx_rx_time_limit: 0,
        hw_target_code: 0x01010000,
        slave_ce_len_min: 5,
        hw_recorrect_en: 1 << 0,
        cca_thresh: 20,
        dup_list_refresh_period: 0,
        scan_backoff_upperlimitmax: 0,
    }
}

pub(crate) unsafe extern "C" fn interrupt_on(intr_num: i32) {
    trace!("interrupt_on {}", intr_num);

    (*hal::pac::INTERRUPT_CORE0::PTR)
        .cpu_int_enable
        .modify(|r, w| w.bits(r.bits() | 1 << intr_num));
}

pub(crate) unsafe extern "C" fn interrupt_off(_intr_num: i32) {
    todo!();
}

pub(crate) fn btdm_controller_mem_init() {
    unsafe {
        btdm_controller_rom_data_init();
    }
}

pub(crate) fn bt_periph_module_enable() {
    // nothing
}

pub(crate) fn disable_sleep_mode() {
    // nothing
}

// OSI functions

pub(crate) unsafe extern "C" fn interrupt_set(
    cpu_no: i32,
    intr_source: i32,
    interrupt_no: i32,
    interrupt_prio: i32,
) {
    trace!(
        "interrupt_set {} {} {} {}",
        cpu_no,
        intr_source,
        interrupt_no,
        interrupt_prio
    );

    ((0x600c2000 + 0x114 + interrupt_no * 4) as *mut u32).write_volatile(interrupt_prio as u32);

    /* Set the interrupt type (Edge or Level). */
    // ----

    /* Map the CPU interrupt ID to the peripheral. */
    ((0x600c2000 + intr_source * 4) as *mut u32).write_volatile(interrupt_no as u32);
}

pub(crate) unsafe extern "C" fn interrupt_clear(_interrupt_source: i32, _interrupt_no: i32) {
    todo!();
}

pub(crate) unsafe extern "C" fn interrupt_handler_set(
    interrupt_no: i32,
    func: extern "C" fn(),
    arg: *const (),
) {
    trace!(
        "interrupt_handler_set {} {:p} {:p}",
        interrupt_no,
        func,
        arg
    );
    match interrupt_no {
        5 => {
            BT_INTERRUPT_FUNCTION5 = (
                func as *mut crate::binary::c_types::c_void,
                arg as *mut crate::binary::c_types::c_void,
            )
        }
        8 => {
            BT_INTERRUPT_FUNCTION8 = (
                func as *mut crate::binary::c_types::c_void,
                arg as *mut crate::binary::c_types::c_void,
            )
        }
        _ => panic!("Unsupported interrupt number {}", interrupt_no),
    }
}

pub(crate) unsafe extern "C" fn coex_wifi_sleep_set(sleep: i32) {
    log::debug!(
        "ignored coex_wifi_sleep_set {} - because original implementation does the same",
        sleep
    );
}

#[allow(unused_variables, dead_code)]
pub(crate) unsafe extern "C" fn coex_core_ble_conn_dyn_prio_get(
    low: *mut i32,
    high: *mut i32,
) -> i32 {
    extern "C" {
        fn coex_core_ble_conn_dyn_prio_get(low: *mut i32, high: *mut i32) -> i32;
    }
    log::debug!("coex_core_ble_conn_dyn_prio_get");

    #[cfg(coex)]
    return coex_core_ble_conn_dyn_prio_get(low, high);

    #[cfg(not(coex))]
    0
}

pub(crate) unsafe extern "C" fn esp_hw_power_down() {
    todo!();
}

pub(crate) unsafe extern "C" fn esp_hw_power_up() {
    todo!();
}

pub(crate) unsafe extern "C" fn ets_backup_dma_copy(
    _reg: u32,
    _mem_addr: u32,
    _num: u32,
    _to_rem: i32,
) {
    todo!();
}
