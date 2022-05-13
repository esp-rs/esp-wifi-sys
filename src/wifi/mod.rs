#[doc(hidden)]
pub mod os_adapter;
#[cfg(feature = "esp32")]
use esp32_hal as hal;
#[cfg(feature = "esp32")]
use esp32_hal::Rng;
#[cfg(feature = "esp32c3")]
use esp32c3_hal as hal;
#[cfg(feature = "esp32c3")]
use esp32c3_hal::Rng;

#[doc(hidden)]
pub use os_adapter::*;
use smoltcp::phy::{Device, DeviceCapabilities, RxToken, TxToken};
#[cfg(feature = "esp32")]
mod phy_init_data_esp32;
#[cfg(feature = "esp32c3")]
mod phy_init_data_esp32c3;

#[cfg(feature = "esp32")]
mod additional_esp32;

#[cfg(feature = "utils")]
pub mod utils;

use crate::{
    binary::include::{
        __BindgenBitfieldUnit, esp_err_t, esp_interface_t_ESP_IF_WIFI_STA, esp_supplicant_init,
        esp_wifi_connect, esp_wifi_init_internal, esp_wifi_internal_free_rx_buffer,
        esp_wifi_internal_reg_rxcb, esp_wifi_internal_tx, esp_wifi_scan_start, esp_wifi_set_config,
        esp_wifi_set_country, esp_wifi_set_mode, esp_wifi_set_ps, esp_wifi_set_tx_done_cb,
        esp_wifi_start, esp_wifi_stop, g_wifi_default_wpa_crypto_funcs, wifi_active_scan_time_t,
        wifi_auth_mode_t_WIFI_AUTH_OPEN, wifi_config_t,
        wifi_country_policy_t_WIFI_COUNTRY_POLICY_MANUAL, wifi_country_t, wifi_init_config_t,
        wifi_interface_t_WIFI_IF_STA, wifi_mode_t_WIFI_MODE_STA, wifi_osi_funcs_t,
        wifi_pmf_config_t, wifi_ps_type_t_WIFI_PS_NONE, wifi_scan_config_t,
        wifi_scan_method_t_WIFI_FAST_SCAN, wifi_scan_threshold_t, wifi_scan_time_t,
        wifi_scan_type_t_WIFI_SCAN_TYPE_ACTIVE, wifi_sort_method_t_WIFI_CONNECT_AP_BY_SIGNAL,
        wifi_sta_config_t, wpa_crypto_funcs_t, ESP_WIFI_OS_ADAPTER_MAGIC,
        ESP_WIFI_OS_ADAPTER_VERSION, WIFI_INIT_CONFIG_MAGIC,
    },
    compat::queue::SimpleQueue,
};
use crate::{tasks::init_tasks, timer::setup_timer_isr};
use log::{debug, info};

#[cfg(feature = "dump_packets")]
static DUMP_PACKETS: bool = true;
#[cfg(not(feature = "dump_packets"))]
static DUMP_PACKETS: bool = false;

struct DataFrame {
    len: usize,
    data: [u8; 2500],
}

static mut DATA_QUEUE_RX: Option<SimpleQueue<DataFrame, 3>> = None;

pub static mut TX_BUFFER: [u8; 2500] = [0u8; 2500]; // should be a queue
pub static mut TX_QUEUED: bool = false;
pub static mut TX_QUEUED_DATA_LEN: u16 = 0;

static mut RANDOM_GENERATOR: Option<Rng> = None;

#[derive(Debug, Clone, Copy)]
pub enum WifiError {
    General(i32),
}

#[allow(unused)]
static mut BLE_ENABLED: bool = false;

#[cfg(feature = "esp32c3")]
/// Initialize for using WiFi
/// This will initialize internals and also initialize WiFi
pub fn initialize(
    systimer: &mut esp32c3_hal::pac::SYSTIMER,
    interrupt_core0: &mut esp32c3_hal::pac::INTERRUPT_CORE0,
    rng: hal::pac::RNG,
) -> Result<(), WifiError> {
    init_rng(rng);
    init_tasks();
    setup_timer_isr(systimer, interrupt_core0);
    wifi_set_log_verbose();
    init_clocks();
    init_buffer();
    let res = wifi_init();
    if res != 0 {
        return Err(WifiError::General(res));
    }
    let res = wifi_start();
    if res != 0 {
        return Err(WifiError::General(res));
    }

    Ok(())
}

#[cfg(feature = "esp32c3")]
/// Initialize for using Bluetooth LE
/// This will just initialize internals.
pub fn initialize_ble(
    systimer: &mut esp32c3_hal::pac::SYSTIMER,
    interrupt_core0: &mut esp32c3_hal::pac::INTERRUPT_CORE0,
    rng: hal::pac::RNG,
) -> Result<(), WifiError> {
    init_rng(rng);
    init_tasks();
    setup_timer_isr(systimer, interrupt_core0);
    wifi_set_log_verbose();
    init_clocks();
    init_buffer();

    unsafe {
        BLE_ENABLED = true;
    }

    Ok(())
}

#[cfg(feature = "esp32")]
/// Initialize for using WiFi
/// This will initialize internals and also initialize WiFi
pub fn initialize(timg1: esp32_hal::pac::TIMG1, rng: hal::pac::RNG) -> Result<(), WifiError> {
    init_rng(rng);
    init_tasks();
    setup_timer_isr(timg1);
    wifi_set_log_verbose();
    init_clocks();
    init_buffer();
    let res = wifi_init();
    if res != 0 {
        return Err(WifiError::General(res));
    }
    let res = wifi_start();
    if res != 0 {
        return Err(WifiError::General(res));
    }

    Ok(())
}

pub fn init_buffer() {
    unsafe {
        DATA_QUEUE_RX = Some(SimpleQueue::new());
    }
}

pub fn init_rng(rng: hal::pac::RNG) {
    unsafe {
        RANDOM_GENERATOR = Some(Rng::new(rng));
    }
}

pub fn init_clocks() {
    crate::wifi::os_adapter::os_adapter_chip_specific::init_clocks();
}

pub fn wifi_set_log_verbose() {
    #[cfg(feature = "wifi_logs")]
    unsafe {
        use crate::binary::include::{esp_wifi_internal_set_log_level, wifi_log_level_t};

        let level: wifi_log_level_t = crate::binary::include::wifi_log_level_t_WIFI_LOG_VERBOSE;
        esp_wifi_internal_set_log_level(level);
    }
}

#[no_mangle]
static g_wifi_osi_funcs: wifi_osi_funcs_t = wifi_osi_funcs_t {
    _version: ESP_WIFI_OS_ADAPTER_VERSION as i32,
    _env_is_chip: Some(env_is_chip),
    _set_intr: Some(set_intr),
    _clear_intr: Some(clear_intr),
    _set_isr: Some(set_isr),
    _ints_on: Some(ints_on),
    _ints_off: Some(ints_off),
    _is_from_isr: Some(is_from_isr),
    _spin_lock_create: Some(spin_lock_create),
    _spin_lock_delete: Some(spin_lock_delete),
    _wifi_int_disable: Some(wifi_int_disable),
    _wifi_int_restore: Some(wifi_int_restore),
    _task_yield_from_isr: Some(task_yield_from_isr),
    _semphr_create: Some(semphr_create),
    _semphr_delete: Some(semphr_delete),
    _semphr_take: Some(semphr_take),
    _semphr_give: Some(semphr_give),
    _wifi_thread_semphr_get: Some(wifi_thread_semphr_get),
    _mutex_create: Some(mutex_create),
    _recursive_mutex_create: Some(recursive_mutex_create),
    _mutex_delete: Some(mutex_delete),
    _mutex_lock: Some(mutex_lock),
    _mutex_unlock: Some(mutex_unlock),
    _queue_create: Some(queue_create),
    _queue_delete: Some(queue_delete),
    _queue_send: Some(queue_send),
    _queue_send_from_isr: Some(queue_send_from_isr),
    _queue_send_to_back: Some(queue_send_to_back),
    _queue_send_to_front: Some(queue_send_to_front),
    _queue_recv: Some(queue_recv),
    _queue_msg_waiting: Some(queue_msg_waiting),
    _event_group_create: Some(event_group_create),
    _event_group_delete: Some(event_group_delete),
    _event_group_set_bits: Some(event_group_set_bits),
    _event_group_clear_bits: Some(event_group_clear_bits),
    _event_group_wait_bits: Some(event_group_wait_bits),
    _task_create_pinned_to_core: Some(task_create_pinned_to_core),
    _task_create: Some(task_create),
    _task_delete: Some(task_delete),
    _task_delay: Some(task_delay),
    _task_ms_to_tick: Some(task_ms_to_tick),
    _task_get_current_task: Some(task_get_current_task),
    _task_get_max_priority: Some(task_get_max_priority),
    _malloc: Some(malloc),
    _free: Some(free),
    _event_post: Some(event_post),
    _get_free_heap_size: Some(get_free_heap_size),
    _rand: Some(rand),
    _dport_access_stall_other_cpu_start_wrap: Some(dport_access_stall_other_cpu_start_wrap),
    _dport_access_stall_other_cpu_end_wrap: Some(dport_access_stall_other_cpu_end_wrap),
    _wifi_apb80m_request: Some(wifi_apb80m_request),
    _wifi_apb80m_release: Some(wifi_apb80m_release),
    _phy_disable: Some(phy_disable),
    _phy_enable: Some(phy_enable),
    _phy_update_country_info: Some(phy_update_country_info),
    _read_mac: Some(read_mac),
    _timer_arm: Some(timer_arm),
    _timer_disarm: Some(timer_disarm),
    _timer_done: Some(timer_done),
    _timer_setfn: Some(timer_setfn),
    _timer_arm_us: Some(timer_arm_us),
    _wifi_reset_mac: Some(wifi_reset_mac),
    _wifi_clock_enable: Some(wifi_clock_enable),
    _wifi_clock_disable: Some(wifi_clock_disable),
    _wifi_rtc_enable_iso: Some(wifi_rtc_enable_iso),
    _wifi_rtc_disable_iso: Some(wifi_rtc_disable_iso),
    _esp_timer_get_time: Some(esp_timer_get_time),
    _nvs_set_i8: Some(nvs_set_i8),
    _nvs_get_i8: Some(nvs_get_i8),
    _nvs_set_u8: Some(nvs_set_u8),
    _nvs_get_u8: Some(nvs_get_u8),
    _nvs_set_u16: Some(nvs_set_u16),
    _nvs_get_u16: Some(nvs_get_u16),
    _nvs_open: Some(nvs_open),
    _nvs_close: Some(nvs_close),
    _nvs_commit: Some(nvs_commit),
    _nvs_set_blob: Some(nvs_set_blob),
    _nvs_get_blob: Some(nvs_get_blob),
    _nvs_erase_key: Some(nvs_erase_key),
    _get_random: Some(get_random),
    _get_time: Some(get_time),
    _random: Some(random),
    _log_write: Some(log_write),
    _log_writev: Some(log_writev),
    _log_timestamp: Some(log_timestamp),
    _malloc_internal: Some(malloc_internal),
    _realloc_internal: Some(realloc_internal),
    _calloc_internal: Some(calloc_internal),
    _zalloc_internal: Some(zalloc_internal),
    _wifi_malloc: Some(wifi_malloc),
    _wifi_realloc: Some(wifi_realloc),
    _wifi_calloc: Some(wifi_calloc),
    _wifi_zalloc: Some(wifi_zalloc),
    _wifi_create_queue: Some(wifi_create_queue),
    _wifi_delete_queue: Some(wifi_delete_queue),
    _coex_init: Some(coex_init),
    _coex_deinit: Some(coex_deinit),
    _coex_enable: Some(coex_enable),
    _coex_disable: Some(coex_disable),
    _coex_status_get: Some(coex_status_get),
    _coex_condition_set: Some(coex_condition_set),
    _coex_wifi_request: Some(coex_wifi_request),
    _coex_wifi_release: Some(coex_wifi_release),
    _coex_wifi_channel_set: Some(coex_wifi_channel_set),
    _coex_event_duration_get: Some(coex_event_duration_get),
    _coex_pti_get: Some(coex_pti_get),
    _coex_schm_status_bit_clear: Some(coex_schm_status_bit_clear),
    _coex_schm_status_bit_set: Some(coex_schm_status_bit_set),
    _coex_schm_interval_set: Some(coex_schm_interval_set),
    _coex_schm_interval_get: Some(coex_schm_interval_get),
    _coex_schm_curr_period_get: Some(coex_schm_curr_period_get),
    _coex_schm_curr_phase_get: Some(coex_schm_curr_phase_get),
    _coex_schm_curr_phase_idx_set: Some(coex_schm_curr_phase_idx_set),
    _coex_schm_curr_phase_idx_get: Some(coex_schm_curr_phase_idx_get),
    #[cfg(feature = "esp32c3")]
    _slowclk_cal_get: Some(slowclk_cal_get),
    #[cfg(feature = "esp32")]
    _phy_common_clock_disable: Some(
        crate::wifi::os_adapter::os_adapter_chip_specific::phy_common_clock_disable,
    ),
    #[cfg(feature = "esp32")]
    _phy_common_clock_enable: Some(
        crate::wifi::os_adapter::os_adapter_chip_specific::phy_common_clock_enable,
    ),
    _magic: ESP_WIFI_OS_ADAPTER_MAGIC as i32,
};

const CONFIG_FEATURE_WPA3_SAE_BIT: u64 = 1 << 0;

unsafe impl Sync for wifi_init_config_t {}
unsafe impl Sync for wifi_osi_funcs_t {}

#[no_mangle]
static mut g_wifi_feature_caps: u64 = CONFIG_FEATURE_WPA3_SAE_BIT;

static mut G_CONFIG: wifi_init_config_t = wifi_init_config_t {
    event_handler: Some(esp_event_send_internal),
    osi_funcs: &g_wifi_osi_funcs as *const _ as *mut _,

    // dummy for now - populated in init
    wpa_crypto_funcs: wpa_crypto_funcs_t {
        size: 0,
        version: 1,
        aes_wrap: None,
        aes_unwrap: None,
        hmac_sha256_vector: None,
        sha256_prf: None,
        hmac_md5: None,
        hamc_md5_vector: None,
        hmac_sha1: None,
        hmac_sha1_vector: None,
        sha1_prf: None,
        sha1_vector: None,
        pbkdf2_sha1: None,
        rc4_skip: None,
        md5_vector: None,
        aes_encrypt: None,
        aes_encrypt_init: None,
        aes_encrypt_deinit: None,
        aes_decrypt: None,
        aes_decrypt_init: None,
        aes_decrypt_deinit: None,
        aes_128_encrypt: None,
        aes_128_decrypt: None,
        omac1_aes_128: None,
        ccmp_decrypt: None,
        ccmp_encrypt: None,
    },
    static_rx_buf_num: 10,
    dynamic_rx_buf_num: 32,
    tx_buf_type: 1, // offset 0x78
    static_tx_buf_num: 0,
    dynamic_tx_buf_num: 32,
    cache_tx_buf_num: 0,
    csi_enable: 1,
    ampdu_rx_enable: 0,
    ampdu_tx_enable: 0,
    amsdu_tx_enable: 0,
    nvs_enable: 0,
    nano_enable: 0,
    rx_ba_win: 6,
    wifi_task_core_id: 0,
    beacon_max_len: 752,
    mgmt_sbuf_num: 32,
    feature_caps: CONFIG_FEATURE_WPA3_SAE_BIT,
    sta_disconnected_pm: false,
    magic: WIFI_INIT_CONFIG_MAGIC as i32,
};

pub fn get_sta_mac(mac: &mut [u8; 6]) {
    unsafe {
        read_mac(mac as *mut u8, 0);
    }
}

pub fn wifi_init() -> i32 {
    unsafe {
        G_CONFIG.wpa_crypto_funcs = g_wifi_default_wpa_crypto_funcs;
        G_CONFIG.feature_caps = g_wifi_feature_caps;

        let cntry_code = [b'C', b'N', 0];
        let country = wifi_country_t {
            cc: cntry_code,
            schan: 1,
            nchan: 13,
            max_tx_power: 20,
            policy: wifi_country_policy_t_WIFI_COUNTRY_POLICY_MANUAL,
        };

        wifi_set_log_verbose();

        let res = esp_wifi_init_internal(&G_CONFIG);
        if res != 0 {
            return res;
        }

        wifi_set_log_verbose();
        let res = esp_supplicant_init();
        if res != 0 {
            return res;
        }

        let res = esp_wifi_set_mode(wifi_mode_t_WIFI_MODE_STA);
        if res != 0 {
            return res;
        }

        let mut cfg = wifi_config_t {
            sta: wifi_sta_config_t {
                ssid: [0; 32],
                password: [0; 64],
                scan_method: wifi_scan_method_t_WIFI_FAST_SCAN,
                bssid_set: false,
                bssid: [0; 6],
                channel: 0,
                listen_interval: 3,
                sort_method: wifi_sort_method_t_WIFI_CONNECT_AP_BY_SIGNAL,
                threshold: wifi_scan_threshold_t {
                    rssi: 20,
                    authmode: wifi_auth_mode_t_WIFI_AUTH_OPEN,
                },
                pmf_cfg: wifi_pmf_config_t {
                    capable: false,
                    required: false,
                },
                _bitfield_align_1: [0u32; 0],
                _bitfield_1: __BindgenBitfieldUnit::new([0u8; 4usize]),
            },
        };
        let res = esp_wifi_set_config(wifi_interface_t_WIFI_IF_STA, &mut cfg);
        if res != 0 {
            return res;
        }

        let res = esp_wifi_set_tx_done_cb(Some(esp_wifi_tx_done_cb));
        if res != 0 {
            return res;
        }

        let res = esp_wifi_set_country(&country);
        if res != 0 {
            return res;
        }

        let res = esp_wifi_internal_reg_rxcb(esp_interface_t_ESP_IF_WIFI_STA, Some(recv_cb));
        if res != 0 {
            return res;
        }

        #[cfg(feature = "esp32")]
        {
            static mut NVS_STRUCT: [u32; 12] = [0; 12];
            additional_esp32::g_misc_nvs = &NVS_STRUCT as *const _ as *const u32 as u32;
        }

        0
    }
}

unsafe extern "C" fn recv_cb(
    buffer: *mut crate::binary::c_types::c_void,
    len: u16,
    eb: *mut crate::binary::c_types::c_void,
) -> esp_err_t {
    critical_section::with(|_| {
        if let Some(ref mut data_queue_rx) = DATA_QUEUE_RX {
            if !data_queue_rx.is_full() {
                let mut buf = [0u8; 2500];
                let src = core::slice::from_raw_parts_mut(buffer as *mut u8, len as usize);
                buf[..(len as usize)].copy_from_slice(src);
                data_queue_rx.enqueue(DataFrame {
                    len: len as usize,
                    data: buf,
                });

                esp_wifi_internal_free_rx_buffer(eb);
                debug!("esp_wifi_internal_free_rx_buffer done");
            }
        }
    });

    0
}

unsafe extern "C" fn esp_wifi_tx_done_cb(
    _ifidx: u8,
    _data: *mut u8,
    _data_len: *mut u16,
    _tx_status: bool,
) {
    debug!("esp_wifi_tx_done_cb");
}

pub fn wifi_start() -> i32 {
    unsafe {
        let res = esp_wifi_start();
        if res != 0 {
            return res;
        }

        let res = esp_wifi_set_ps(wifi_ps_type_t_WIFI_PS_NONE);
        if res != 0 {
            return res;
        }
    }

    0
}

pub fn wifi_start_scan() -> i32 {
    let scan_time = wifi_scan_time_t {
        active: wifi_active_scan_time_t { min: 0, max: 0 },
        passive: 0,
    };

    let scan_config = wifi_scan_config_t {
        ssid: core::ptr::null_mut(),
        bssid: core::ptr::null_mut(),
        channel: 0,
        show_hidden: false,
        scan_type: wifi_scan_type_t_WIFI_SCAN_TYPE_ACTIVE,
        scan_time: scan_time,
    };

    unsafe { esp_wifi_scan_start(&scan_config, true) }
}

pub fn wifi_connect(ssid: &str, password: &str) -> i32 {
    unsafe {
        let mut cfg = wifi_config_t {
            sta: wifi_sta_config_t {
                ssid: [0; 32],
                password: [0; 64],
                scan_method: wifi_scan_method_t_WIFI_FAST_SCAN,
                bssid_set: false,
                bssid: [0; 6],
                channel: 0,
                listen_interval: 3,
                sort_method: wifi_sort_method_t_WIFI_CONNECT_AP_BY_SIGNAL,
                threshold: wifi_scan_threshold_t {
                    rssi: -99,
                    authmode: wifi_auth_mode_t_WIFI_AUTH_OPEN,
                },
                pmf_cfg: wifi_pmf_config_t {
                    capable: true,
                    required: false,
                },
                _bitfield_align_1: [0u32; 0],
                _bitfield_1: __BindgenBitfieldUnit::new([0u8; 4usize]),
            },
        };

        cfg.sta.ssid[0..(ssid.len())].copy_from_slice(ssid.as_bytes());
        cfg.sta.password[0..(password.len())].copy_from_slice(password.as_bytes());

        let res = esp_wifi_set_config(wifi_interface_t_WIFI_IF_STA, &mut cfg);
        if res != 0 {
            return res;
        }

        esp_wifi_connect()
    }
}

pub fn wifi_stop() -> i32 {
    unsafe { esp_wifi_stop() }
}

/// A wifi device implementing smoltcp's Device trait.
pub struct WifiDevice {}

impl WifiDevice {
    pub fn new() -> WifiDevice {
        WifiDevice {}
    }
}

// see https://docs.rs/smoltcp/0.7.1/smoltcp/phy/index.html
impl<'a> Device<'a> for WifiDevice {
    type RxToken = WifiRxToken;

    type TxToken = WifiTxToken;

    fn receive(&'a mut self) -> Option<(Self::RxToken, Self::TxToken)> {
        let available = unsafe {
            if let Some(ref data_queue_rx) = DATA_QUEUE_RX {
                !data_queue_rx.is_empty()
            } else {
                false
            }
        };

        if available {
            Some((WifiRxToken::default(), WifiTxToken::default()))
        } else {
            None
        }
    }

    fn transmit(&'a mut self) -> Option<Self::TxToken> {
        Some(WifiTxToken::default())
    }

    fn capabilities(&self) -> smoltcp::phy::DeviceCapabilities {
        let mut caps = DeviceCapabilities::default();
        caps.max_transmission_unit = 1514;
        caps.max_burst_size = Some(1);
        caps
    }
}

#[derive(Debug, Default)]
pub struct WifiRxToken {}

impl RxToken for WifiRxToken {
    fn consume<R, F>(self, _timestamp: smoltcp::time::Instant, f: F) -> smoltcp::Result<R>
    where
        F: FnOnce(&mut [u8]) -> smoltcp::Result<R>,
    {
        let mut result: Option<smoltcp::Result<R>> = None;
        unsafe {
            if let Some(ref mut data_queue_rx) = DATA_QUEUE_RX {
                if !data_queue_rx.is_empty() {
                    let element = data_queue_rx.dequeue();

                    result = match element {
                        Some(mut data) => {
                            let buffer =
                                core::slice::from_raw_parts(&data.data as *const u8, data.len);
                            debug!("received {:?}", _timestamp);
                            dump_packet_info(&buffer);
                            Some(f(&mut data.data[..]))
                        }
                        None => Some(Err(smoltcp::Error::Exhausted)),
                    };
                }
            }
        }

        if let Some(res) = result {
            res
        } else {
            Err(smoltcp::Error::Exhausted)
        }
    }
}

#[derive(Debug, Default)]
pub struct WifiTxToken {}

impl TxToken for WifiTxToken {
    fn consume<R, F>(
        self,
        _timestamp: smoltcp::time::Instant,
        len: usize,
        f: F,
    ) -> smoltcp::Result<R>
    where
        F: FnOnce(&mut [u8]) -> smoltcp::Result<R>,
    {
        let res = unsafe { f(&mut TX_BUFFER[..len]) };

        match res {
            Ok(_) => critical_section::with(|_| unsafe {
                if !TX_QUEUED {
                    TX_QUEUED_DATA_LEN = len as u16;
                    TX_QUEUED = true;
                    res
                } else {
                    Err(smoltcp::Error::Exhausted)
                }
            }),
            Err(_) => res,
        }
    }
}

pub fn send_data_if_needed() {
    let to_send = critical_section::with(|_| unsafe {
        if TX_QUEUED {
            debug!("sending... {} bytes", TX_QUEUED_DATA_LEN);
            dump_packet_info(&TX_BUFFER);
            TX_QUEUED = false;
            Some((TX_BUFFER, TX_QUEUED_DATA_LEN))
        } else {
            None
        }
    });

    if let Some((data, len)) = to_send {
        unsafe {
            let _res = esp_wifi_internal_tx(
                wifi_interface_t_WIFI_IF_STA,
                &data as *const _ as *mut crate::binary::c_types::c_void,
                len,
            );
            debug!("esp_wifi_internal_tx {}", _res);
        }
    }
}

fn dump_packet_info(buffer: &[u8]) {
    if !DUMP_PACKETS {
        return;
    }

    let ef = smoltcp::wire::EthernetFrame::new_unchecked(buffer);
    info!(
        "src={:x?} dst={:x?} type={:x?}",
        ef.src_addr(),
        ef.dst_addr(),
        ef.ethertype()
    );
    match ef.ethertype() {
        smoltcp::wire::EthernetProtocol::Ipv4 => {
            let ip = smoltcp::wire::Ipv4Packet::new_unchecked(ef.payload());
            info!(
                "src={:?} dst={:?} proto={:x?}",
                ip.src_addr(),
                ip.dst_addr(),
                ip.protocol()
            );

            match ip.protocol() {
                smoltcp::wire::IpProtocol::HopByHop => {}
                smoltcp::wire::IpProtocol::Icmp => {}
                smoltcp::wire::IpProtocol::Igmp => {}
                smoltcp::wire::IpProtocol::Tcp => {
                    let tp = smoltcp::wire::TcpPacket::new_unchecked(ip.payload());
                    info!("src={:?} dst={:?}", tp.src_port(), tp.dst_port());
                }
                smoltcp::wire::IpProtocol::Udp => {
                    let up = smoltcp::wire::UdpPacket::new_unchecked(ip.payload());
                    info!("src={:?} dst={:?}", up.src_port(), up.dst_port());
                }
                smoltcp::wire::IpProtocol::Ipv6Route => {}
                smoltcp::wire::IpProtocol::Ipv6Frag => {}
                smoltcp::wire::IpProtocol::Icmpv6 => {}
                smoltcp::wire::IpProtocol::Ipv6NoNxt => {}
                smoltcp::wire::IpProtocol::Ipv6Opts => {}
                smoltcp::wire::IpProtocol::Unknown(_) => {}
            }
        }
        smoltcp::wire::EthernetProtocol::Arp => {
            let ap = smoltcp::wire::ArpPacket::new_unchecked(ef.payload());
            info!(
                "src={:x?} dst={:x?} src proto addr={:x?}",
                ap.source_hardware_addr(),
                ap.target_hardware_addr(),
                ap.source_protocol_addr()
            );
        }
        smoltcp::wire::EthernetProtocol::Ipv6 => {}
        smoltcp::wire::EthernetProtocol::Unknown(_) => {}
    }
}
