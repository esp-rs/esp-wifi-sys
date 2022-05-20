use core::mem::MaybeUninit;
use log::trace;
use Option;

use crate::compat::common::StrBuf;
use crate::compat::queue::SimpleQueue;

use crate::binary::include::*;
use crate::compat::work_queue::queue_work;
use esp_alloc::memory_fence;

#[cfg_attr(feature = "esp32c3", path = "os_adapter_esp32c3.rs")]
#[cfg_attr(feature = "esp32", path = "os_adapter_esp32.rs")]
pub(crate) mod ble_os_adapter_chip_specific;

pub mod controller;

static mut BLE_INITIALIZED: bool = false;

static mut BT_RECEIVE_QUEUE: Option<SimpleQueue<ReceivedPacket, 10>> = None;

pub struct ReceivedPacket {
    pub len: u8,
    pub data: [u8; 256],
}

static mut BT_INTERNAL_QUEUE: Option<SimpleQueue<[u8; 8], 5>> = None;

#[repr(C)]
struct vhci_host_callback_s {
    notify_host_send_available: extern "C" fn(), /* callback used to notify that the host can send packet to controller */
    notify_host_recv: extern "C" fn(*mut u8, u16) -> i32, /* callback used to notify that the controller has a packet to send to the host */
}

extern "C" {
    fn btdm_osi_funcs_register(osi_funcs: *const ()) -> i32;
    fn btdm_controller_get_compile_version() -> *const u8;

    #[cfg(feature = "esp32c3")]
    fn btdm_controller_init(config_opts: *const esp_bt_controller_config_t) -> i32;

    #[cfg(feature = "esp32")]
    fn btdm_controller_init(
        config_mask: u32,
        config_opts: *const esp_bt_controller_config_t,
    ) -> i32;

    fn btdm_controller_enable(mode: esp_bt_mode_t);

    fn API_vhci_host_check_send_available() -> bool;
    fn API_vhci_host_send_packet(data: *const u8, len: u16);
    fn API_vhci_host_register_callback(vhci_host_callbac: *const vhci_host_callback_s) -> i32;
}

static VHCI_HOST_CALLBACK: vhci_host_callback_s = vhci_host_callback_s {
    notify_host_send_available: notify_host_send_available,
    notify_host_recv: notify_host_recv,
};

extern "C" fn notify_host_send_available() {
    trace!("notify_host_send_available");
}

extern "C" fn notify_host_recv(data: *mut u8, len: u16) -> i32 {
    trace!("notify_host_recv {:p} {}", data, len);

    unsafe {
        let mut buf = [0u8; 256];
        for i in 0..len {
            let b = data.offset(i as isize).read();
            buf[i as usize] = b;
        }

        let packet = ReceivedPacket {
            len: len as u8,
            data: buf,
        };

        BT_RECEIVE_QUEUE.as_mut().unwrap().enqueue(packet);
    }

    0
}

#[cfg(feature = "esp32c3")]
#[repr(C)]
struct osi_funcs_s {
    magic: u32,
    version: u32,
    interrupt_set: Option<unsafe extern "C" fn(i32, i32, i32, i32) -> ()>,
    interrupt_clear: Option<unsafe extern "C" fn(i32, i32) -> ()>,
    interrupt_handler_set: Option<unsafe extern "C" fn(i32, extern "C" fn(), *const ()) -> ()>,
    interrupt_disable: Option<unsafe extern "C" fn() -> ()>,
    interrupt_enable: Option<unsafe extern "C" fn() -> ()>,
    task_yield: Option<unsafe extern "C" fn() -> ()>,
    task_yield_from_isr: Option<unsafe extern "C" fn() -> ()>,
    semphr_create: Option<unsafe extern "C" fn(u32, u32) -> *const ()>,
    semphr_delete: Option<unsafe extern "C" fn(*const ()) -> ()>,
    semphr_take_from_isr: Option<unsafe extern "C" fn(*const (), *const ()) -> i32>,
    semphr_give_from_isr: Option<unsafe extern "C" fn(*const (), *const ()) -> i32>,
    semphr_take: Option<unsafe extern "C" fn(*const (), u32) -> i32>,
    semphr_give: Option<unsafe extern "C" fn(*const ()) -> i32>,
    mutex_create: Option<unsafe extern "C" fn() -> *const ()>,
    mutex_delete: Option<unsafe extern "C" fn(*const ()) -> ()>,
    mutex_lock: Option<unsafe extern "C" fn(*const ()) -> i32>,
    mutex_unlock: Option<unsafe extern "C" fn(*const ()) -> i32>,
    queue_create: Option<unsafe extern "C" fn(u32, u32) -> *const ()>,
    queue_delete: Option<unsafe extern "C" fn(*const ()) -> ()>,
    queue_send: Option<unsafe extern "C" fn(*const (), *const (), u32) -> i32>,
    queue_send_from_isr: Option<unsafe extern "C" fn(*const (), *const (), *const ()) -> i32>,
    queue_recv: Option<unsafe extern "C" fn(*const (), *const (), u32) -> i32>,
    queue_recv_from_isr: Option<unsafe extern "C" fn(*const (), *const (), *const ()) -> i32>,
    task_create: Option<
        unsafe extern "C" fn(
            *mut crate::binary::c_types::c_void,
            *const u8,
            u32,
            *mut crate::binary::c_types::c_void,
            u32,
            *mut crate::binary::c_types::c_void,
            u32,
        ) -> i32,
    >,
    task_delete: Option<unsafe extern "C" fn(*const ()) -> ()>,
    is_in_isr: Option<unsafe extern "C" fn() -> i32>,
    cause_sw_intr_to_core: Option<unsafe extern "C" fn(i32, i32) -> i32>,
    malloc: Option<unsafe extern "C" fn(u32) -> *const ()>,
    malloc_internal: Option<unsafe extern "C" fn(u32) -> *const ()>,
    free: Option<unsafe extern "C" fn(*const ()) -> ()>,
    read_efuse_mac: Option<unsafe extern "C" fn(*const ()) -> i32>,
    srand: Option<unsafe extern "C" fn(u32) -> ()>,
    rand: Option<unsafe extern "C" fn() -> i32>,
    btdm_lpcycles_2_hus: Option<unsafe extern "C" fn(u32, u32) -> u32>,
    btdm_hus_2_lpcycles: Option<unsafe extern "C" fn(u32) -> u32>,
    btdm_sleep_check_duration: Option<unsafe extern "C" fn(i32) -> i32>,
    btdm_sleep_enter_phase1: Option<unsafe extern "C" fn(i32) -> ()>,
    btdm_sleep_enter_phase2: Option<unsafe extern "C" fn() -> ()>,
    btdm_sleep_exit_phase1: Option<unsafe extern "C" fn() -> ()>,
    btdm_sleep_exit_phase2: Option<unsafe extern "C" fn() -> ()>,
    btdm_sleep_exit_phase3: Option<unsafe extern "C" fn() -> ()>,
    coex_wifi_sleep_set: Option<unsafe extern "C" fn(i32) -> ()>,
    coex_core_ble_conn_dyn_prio_get: Option<unsafe extern "C" fn(*mut i32, *mut i32) -> i32>,
    coex_schm_status_bit_set: Option<unsafe extern "C" fn(i32, i32) -> ()>,
    coex_schm_status_bit_clear: Option<unsafe extern "C" fn(i32, i32) -> ()>,
    interrupt_on: Option<unsafe extern "C" fn(i32) -> ()>,
    interrupt_off: Option<unsafe extern "C" fn(i32) -> ()>,
    esp_hw_power_down: Option<unsafe extern "C" fn() -> ()>,
    esp_hw_power_up: Option<unsafe extern "C" fn() -> ()>,
    ets_backup_dma_copy: Option<unsafe extern "C" fn(u32, u32, u32, i32) -> ()>,
}

#[cfg(feature = "esp32c3")]
static G_OSI_FUNCS: osi_funcs_s = osi_funcs_s {
    magic: 0xfadebead,
    version: 0x00010006,
    interrupt_set: Some(ble_os_adapter_chip_specific::interrupt_set),
    interrupt_clear: Some(ble_os_adapter_chip_specific::interrupt_clear),
    interrupt_handler_set: Some(ble_os_adapter_chip_specific::interrupt_handler_set),
    interrupt_disable: Some(interrupt_disable),
    interrupt_enable: Some(interrupt_enable),
    task_yield: Some(task_yield),
    task_yield_from_isr: Some(task_yield_from_isr),
    semphr_create: Some(semphr_create),
    semphr_delete: Some(semphr_delete),
    semphr_take_from_isr: Some(semphr_take_from_isr),
    semphr_give_from_isr: Some(semphr_give_from_isr),
    semphr_take: Some(semphr_take),
    semphr_give: Some(semphr_give),
    mutex_create: Some(mutex_create),
    mutex_delete: Some(mutex_delete),
    mutex_lock: Some(mutex_lock),
    mutex_unlock: Some(mutex_unlock),
    queue_create: Some(queue_create),
    queue_delete: Some(queue_delete),
    queue_send: Some(queue_send),
    queue_send_from_isr: Some(queue_send_from_isr),
    queue_recv: Some(queue_recv),
    queue_recv_from_isr: Some(queue_recv_from_isr),
    task_create: Some(task_create),
    task_delete: Some(task_delete),
    is_in_isr: Some(is_in_isr),
    cause_sw_intr_to_core: Some(cause_sw_intr_to_core),
    malloc: Some(malloc),
    malloc_internal: Some(malloc_internal),
    free: Some(free),
    read_efuse_mac: Some(read_efuse_mac),
    srand: Some(srand),
    rand: Some(rand),
    btdm_lpcycles_2_hus: Some(btdm_lpcycles_2_hus),
    btdm_hus_2_lpcycles: Some(btdm_hus_2_lpcycles),
    btdm_sleep_check_duration: Some(btdm_sleep_check_duration),
    btdm_sleep_enter_phase1: Some(btdm_sleep_enter_phase1),
    btdm_sleep_enter_phase2: Some(btdm_sleep_enter_phase2),
    btdm_sleep_exit_phase1: Some(btdm_sleep_exit_phase1),
    btdm_sleep_exit_phase2: Some(btdm_sleep_exit_phase2),
    btdm_sleep_exit_phase3: Some(btdm_sleep_exit_phase3),
    coex_wifi_sleep_set: Some(ble_os_adapter_chip_specific::coex_wifi_sleep_set),
    coex_core_ble_conn_dyn_prio_get: Some(
        ble_os_adapter_chip_specific::coex_core_ble_conn_dyn_prio_get,
    ),
    coex_schm_status_bit_set: Some(coex_schm_status_bit_set),
    coex_schm_status_bit_clear: Some(coex_schm_status_bit_clear),
    interrupt_on: Some(ble_os_adapter_chip_specific::interrupt_on),
    interrupt_off: Some(ble_os_adapter_chip_specific::interrupt_off),
    esp_hw_power_down: Some(ble_os_adapter_chip_specific::esp_hw_power_down),
    esp_hw_power_up: Some(ble_os_adapter_chip_specific::esp_hw_power_up),
    ets_backup_dma_copy: Some(ble_os_adapter_chip_specific::ets_backup_dma_copy),
};

#[cfg(feature = "esp32")]
#[repr(C)]
struct osi_funcs_s {
    version: u32,
    set_isr: Option<unsafe extern "C" fn(i32, unsafe extern "C" fn(), *const ()) -> i32>,
    ints_on: Option<unsafe extern "C" fn(u32)>,
    interrupt_disable: Option<unsafe extern "C" fn() -> ()>,
    interrupt_restore: Option<unsafe extern "C" fn() -> ()>,
    task_yield: Option<unsafe extern "C" fn() -> ()>,
    task_yield_from_isr: Option<unsafe extern "C" fn() -> ()>,
    semphr_create: Option<unsafe extern "C" fn(u32, u32) -> *const ()>,
    semphr_delete: Option<unsafe extern "C" fn(*const ()) -> ()>,
    semphr_take_from_isr: Option<unsafe extern "C" fn(*const (), *const ()) -> i32>,
    semphr_give_from_isr: Option<unsafe extern "C" fn(*const (), *const ()) -> i32>,
    semphr_take: Option<unsafe extern "C" fn(*const (), u32) -> i32>,
    semphr_give: Option<unsafe extern "C" fn(*const ()) -> i32>,
    mutex_create: Option<unsafe extern "C" fn() -> *const ()>,
    mutex_delete: Option<unsafe extern "C" fn(*const ()) -> ()>,
    mutex_lock: Option<unsafe extern "C" fn(*const ()) -> i32>,
    mutex_unlock: Option<unsafe extern "C" fn(*const ()) -> i32>,
    queue_create: Option<unsafe extern "C" fn(u32, u32) -> *const ()>,
    queue_delete: Option<unsafe extern "C" fn(*const ()) -> ()>,
    queue_send: Option<unsafe extern "C" fn(*const (), *const (), u32) -> i32>,
    queue_send_from_isr: Option<unsafe extern "C" fn(*const (), *const (), *const ()) -> i32>,
    queue_recv: Option<unsafe extern "C" fn(*const (), *const (), u32) -> i32>,
    queue_recv_from_isr: Option<unsafe extern "C" fn(*const (), *const (), *const ()) -> i32>,
    task_create: Option<
        unsafe extern "C" fn(
            *mut crate::binary::c_types::c_void,
            *const u8,
            u32,
            *mut crate::binary::c_types::c_void,
            u32,
            *mut crate::binary::c_types::c_void,
            u32,
        ) -> i32,
    >,
    task_delete: Option<unsafe extern "C" fn(*const ()) -> ()>,
    is_in_isr: Option<unsafe extern "C" fn() -> i32>,
    cause_sw_intr_to_core: Option<unsafe extern "C" fn(i32, i32) -> i32>,
    malloc: Option<unsafe extern "C" fn(u32) -> *const ()>,
    malloc_internal: Option<unsafe extern "C" fn(u32) -> *const ()>,
    free: Option<unsafe extern "C" fn(*const ()) -> ()>,
    read_efuse_mac: Option<unsafe extern "C" fn(*const ()) -> i32>,
    srand: Option<unsafe extern "C" fn(u32) -> ()>,
    rand: Option<unsafe extern "C" fn() -> i32>,
    btdm_lpcycles_2_hus: Option<unsafe extern "C" fn(u32, u32) -> u32>,
    btdm_hus_2_lpcycles: Option<unsafe extern "C" fn(u32) -> u32>,
    btdm_sleep_check_duration: Option<unsafe extern "C" fn(i32) -> i32>,
    btdm_sleep_enter_phase1: Option<unsafe extern "C" fn(i32) -> ()>,
    btdm_sleep_enter_phase2: Option<unsafe extern "C" fn() -> ()>,
    btdm_sleep_exit_phase1: Option<unsafe extern "C" fn() -> ()>,
    btdm_sleep_exit_phase2: Option<unsafe extern "C" fn() -> ()>,
    btdm_sleep_exit_phase3: Option<unsafe extern "C" fn() -> ()>,
    coex_bt_wakeup_request: Option<unsafe extern "C" fn() -> bool>,
    coex_bt_wakeup_request_end: Option<unsafe extern "C" fn() -> ()>,
    coex_bt_request: Option<unsafe extern "C" fn(u32, u32, u32) -> i32>,
    coex_bt_release: Option<unsafe extern "C" fn(u32) -> i32>,
    coex_register_bt_cb: Option<unsafe extern "C" fn(unsafe extern "C" fn()) -> i32>,
    coex_bb_reset_lock: Option<unsafe extern "C" fn() -> u32>,
    coex_bb_reset_unlock: Option<unsafe extern "C" fn(u32)>,
    coex_schm_register_btdm_callback: Option<unsafe extern "C" fn(unsafe extern "C" fn()) -> i32>,
    coex_schm_status_bit_clear: Option<unsafe extern "C" fn(i32, i32) -> ()>,
    coex_schm_status_bit_set: Option<unsafe extern "C" fn(i32, i32) -> ()>,
    coex_schm_interval_get: Option<unsafe extern "C" fn() -> u32>,
    coex_schm_curr_period_get: Option<unsafe extern "C" fn() -> u8>,
    coex_schm_curr_phase_get: Option<unsafe extern "C" fn() -> *const ()>,
    coex_wifi_channel_get: Option<unsafe extern "C" fn(*mut u8, *mut u8) -> i32>,
    coex_register_wifi_channel_change_callback:
        Option<unsafe extern "C" fn(unsafe extern "C" fn()) -> i32>,
    magic: u32,
}

#[cfg(feature = "esp32")]
static G_OSI_FUNCS: osi_funcs_s = osi_funcs_s {
    version: 0x00010002,
    set_isr: Some(ble_os_adapter_chip_specific::set_isr),
    ints_on: Some(ble_os_adapter_chip_specific::ints_on),
    interrupt_disable: Some(interrupt_disable),
    interrupt_restore: Some(interrupt_enable),
    task_yield: Some(task_yield),
    task_yield_from_isr: Some(task_yield_from_isr),
    semphr_create: Some(semphr_create),
    semphr_delete: Some(semphr_delete),
    semphr_take_from_isr: Some(semphr_take_from_isr),
    semphr_give_from_isr: Some(semphr_give_from_isr),
    semphr_take: Some(semphr_take),
    semphr_give: Some(semphr_give),
    mutex_create: Some(mutex_create),
    mutex_delete: Some(mutex_delete),
    mutex_lock: Some(mutex_lock),
    mutex_unlock: Some(mutex_unlock),
    queue_create: Some(queue_create),
    queue_delete: Some(queue_delete),
    queue_send: Some(queue_send),
    queue_send_from_isr: Some(queue_send_from_isr),
    queue_recv: Some(queue_recv),
    queue_recv_from_isr: Some(queue_recv_from_isr),
    task_create: Some(task_create),
    task_delete: Some(task_delete),
    is_in_isr: Some(is_in_isr),
    cause_sw_intr_to_core: Some(cause_sw_intr_to_core),
    malloc: Some(malloc),
    malloc_internal: Some(malloc_internal),
    free: Some(free),
    read_efuse_mac: Some(read_efuse_mac),
    srand: Some(srand),
    rand: Some(rand),
    btdm_lpcycles_2_hus: Some(btdm_lpcycles_2_hus),
    btdm_hus_2_lpcycles: Some(btdm_hus_2_lpcycles),
    btdm_sleep_check_duration: Some(btdm_sleep_check_duration),
    btdm_sleep_enter_phase1: Some(btdm_sleep_enter_phase1),
    btdm_sleep_enter_phase2: Some(btdm_sleep_enter_phase2),
    btdm_sleep_exit_phase1: Some(btdm_sleep_exit_phase1),
    btdm_sleep_exit_phase2: Some(btdm_sleep_exit_phase2),
    btdm_sleep_exit_phase3: Some(btdm_sleep_exit_phase3),
    coex_bt_wakeup_request: Some(ble_os_adapter_chip_specific::coex_bt_wakeup_request),
    coex_bt_wakeup_request_end: Some(ble_os_adapter_chip_specific::coex_bt_wakeup_request_end),
    coex_bt_request: Some(ble_os_adapter_chip_specific::coex_bt_request),
    coex_bt_release: Some(ble_os_adapter_chip_specific::coex_bt_release),
    coex_register_bt_cb: Some(ble_os_adapter_chip_specific::coex_register_bt_cb),
    coex_bb_reset_lock: Some(ble_os_adapter_chip_specific::coex_bb_reset_lock),
    coex_bb_reset_unlock: Some(ble_os_adapter_chip_specific::coex_bb_reset_unlock),
    coex_schm_register_btdm_callback: Some(
        ble_os_adapter_chip_specific::coex_schm_register_btdm_callback,
    ),
    coex_schm_status_bit_clear: Some(coex_schm_status_bit_clear),
    coex_schm_status_bit_set: Some(coex_schm_status_bit_set),
    coex_schm_interval_get: Some(ble_os_adapter_chip_specific::coex_schm_interval_get),
    coex_schm_curr_period_get: Some(ble_os_adapter_chip_specific::coex_schm_curr_period_get),
    coex_schm_curr_phase_get: Some(ble_os_adapter_chip_specific::coex_schm_curr_phase_get),
    coex_wifi_channel_get: Some(ble_os_adapter_chip_specific::coex_wifi_channel_get),
    coex_register_wifi_channel_change_callback: Some(
        ble_os_adapter_chip_specific::coex_register_wifi_channel_change_callback,
    ),
    magic: 0xfadebead,
};

unsafe extern "C" fn interrupt_enable() {
    trace!("!!!! unimplemented interrupt_enable");
}

unsafe extern "C" fn interrupt_disable() {
    trace!("!!!! unimplemented interrupt_disable");
}

unsafe extern "C" fn task_yield() {
    todo!();
}

unsafe extern "C" fn task_yield_from_isr() {
    todo!();
}

unsafe extern "C" fn semphr_create(max: u32, init: u32) -> *const () {
    crate::wifi::semphr_create(max, init) as *const ()
}

unsafe extern "C" fn semphr_delete(sem: *const ()) {
    crate::wifi::semphr_delete(sem as *mut crate::binary::c_types::c_void);
}

unsafe extern "C" fn semphr_take_from_isr(sem: *const (), _hptw: *const ()) -> i32 {
    trace!("sem take from isr");
    crate::wifi::semphr_take(sem as *mut crate::binary::c_types::c_void, 0)
}

unsafe extern "C" fn semphr_give_from_isr(sem: *const (), _hptw: *const ()) -> i32 {
    trace!("sem give from isr");
    crate::wifi::semphr_give(sem as *mut crate::binary::c_types::c_void)
}

unsafe extern "C" fn semphr_take(sem: *const (), block_time_ms: u32) -> i32 {
    crate::wifi::semphr_take(sem as *mut crate::binary::c_types::c_void, block_time_ms)
}

unsafe extern "C" fn semphr_give(sem: *const ()) -> i32 {
    crate::wifi::semphr_give(sem as *mut crate::binary::c_types::c_void)
}

unsafe extern "C" fn mutex_create() -> *const () {
    todo!();
}

unsafe extern "C" fn mutex_delete(_mutex: *const ()) {
    todo!();
}

unsafe extern "C" fn mutex_lock(_mutex: *const ()) -> i32 {
    todo!();
}

unsafe extern "C" fn mutex_unlock(_mutex: *const ()) -> i32 {
    todo!();
}

unsafe extern "C" fn queue_create(len: u32, item_size: u32) -> *const () {
    if len != 5 && item_size != 8 {
        panic!("Unexpected queue spec {} {}", len, item_size);
    }
    &BT_INTERNAL_QUEUE as *const _ as *const ()
}

unsafe extern "C" fn queue_delete(queue: *const ()) {
    trace!("Unimplemented queue_delete {:p}", queue);
}

unsafe extern "C" fn queue_send(queue: *const (), item: *const (), _block_time_ms: u32) -> i32 {
    if queue == &BT_INTERNAL_QUEUE as *const _ as *const () {
        critical_section::with(|_| {
            // assume the size is 8 - shouldn't rely on that
            let message = item as *const u8;
            let mut data = [0u8; 8];
            for i in 0..8 as usize {
                data[i] = *(message.offset(i as isize));
            }
            trace!("queue posting {:x?}", data);

            BT_INTERNAL_QUEUE.as_mut().unwrap().enqueue(data);
            memory_fence();
        });
    } else {
        panic!("Unknown queue");
    }
    1
}

unsafe extern "C" fn queue_send_from_isr(
    _queue: *const (),
    _item: *const (),
    _hptw: *const (),
) -> i32 {
    log::trace!("queue_send_from_isr {:p} {:p} {:p}", _queue, _item, _hptw);
    // Force to set the value to be false
    *(_hptw as *mut bool) = false;
    queue_send(_queue, _item, 0)
}

unsafe extern "C" fn queue_recv(queue: *const (), item: *const (), block_time_ms: u32) -> i32 {
    trace!(
        "queue_recv {:p} item {:p} block_time_tick {}",
        queue,
        item,
        block_time_ms
    );

    let end_time = crate::timer::get_systimer_count() + block_time_ms as u64;

    // handle the BT_QUEUE
    if queue == &mut BT_INTERNAL_QUEUE as *const _ as *const () {
        loop {
            let res = critical_section::with(|_| {
                memory_fence();
                let message = BT_INTERNAL_QUEUE.as_mut().unwrap().dequeue();
                if message.is_some() {
                    let message = message.unwrap();
                    let item = item as *mut u8;
                    for i in 0..8 {
                        item.offset(i).write_volatile(message[i as usize]);
                    }
                    trace!("received {:x?}", message);
                    1
                } else {
                    0
                }
            });

            if res == 1 {
                trace!("queue_recv returns");
                return res;
            }

            if block_time_ms != OSI_FUNCS_TIME_BLOCKING
                && crate::timer::get_systimer_count() > end_time
            {
                trace!("queue_recv returns with timeout");
                return -1;
            }
        }
    } else {
        panic!("Unknown queue to handle in queue_recv");
    }
}

unsafe extern "C" fn queue_recv_from_isr(
    _queue: *const (),
    _item: *const (),
    _hptw: *const (),
) -> i32 {
    todo!();
}

unsafe extern "C" fn task_create(
    func: *mut crate::binary::c_types::c_void,
    name: *const u8,
    stack_depth: u32,
    param: *mut crate::binary::c_types::c_void,
    prio: u32,
    handle: *mut crate::binary::c_types::c_void,
    core_id: u32,
) -> i32 {
    let n = StrBuf::from(name);
    trace!(
        "recheck implementation: task_create {:p} {:p} {} {} {:p} {} {:p} {}",
        func,
        name,
        n.as_str_ref(),
        stack_depth,
        param,
        prio,
        handle,
        core_id
    );

    *(handle as *mut usize) = 0; // we will run it in task 0

    queue_work(func, name, stack_depth, param, prio, handle, core_id);
    1
}

unsafe extern "C" fn task_delete(_task: *const ()) {
    todo!();
}

unsafe extern "C" fn is_in_isr() -> i32 {
    0
}

unsafe extern "C" fn cause_sw_intr_to_core(_core: i32, _intr_no: i32) -> i32 {
    #[cfg(feature = "esp32c3")]
    todo!("cause_sw_intr_to_core is not implemented for ESP32C3");

    #[cfg(feature = "esp32")]
    {
        log::trace!("cause_sw_intr_to_core {} {}", _core, _intr_no);
        let intr = 1 << _intr_no;
        core::arch::asm!("wsr.226  {0}", in(reg) intr, options(nostack)); // 226 = "intset"
        0
    }
}

unsafe extern "C" fn malloc(size: u32) -> *const () {
    esp_alloc::malloc(size) as *const ()
}

unsafe extern "C" fn malloc_internal(size: u32) -> *const () {
    esp_alloc::malloc(size) as *const ()
}

unsafe extern "C" fn free(ptr: *const ()) {
    esp_alloc::free(ptr as *const u8);
}

unsafe extern "C" fn srand(seed: u32) {
    trace!("!!!! unimplemented srand {}", seed);
}

unsafe extern "C" fn rand() -> i32 {
    trace!("rand");
    crate::wifi::os_adapter::random() as i32
}

unsafe extern "C" fn btdm_lpcycles_2_hus(_cycles: u32, _error_corr: u32) -> u32 {
    todo!();
}

unsafe extern "C" fn btdm_hus_2_lpcycles(us: u32) -> u32 {
    const RTC_CLK_CAL_FRACT: u32 = 19;
    let g_btdm_lpcycle_us_frac = RTC_CLK_CAL_FRACT;
    let g_btdm_lpcycle_us = 2 << (g_btdm_lpcycle_us_frac);

    // Converts a duration in half us into a number of low power clock cycles.
    let cycles: u64 = (us as u64) << (g_btdm_lpcycle_us_frac as u64 / g_btdm_lpcycle_us as u64);
    log::warn!("*** NOT implemented btdm_hus_2_lpcycles {} {}", us, cycles);
    // probably not right ... NX returns half of the values we calculate here

    cycles as u32
}

unsafe extern "C" fn btdm_sleep_check_duration(_slot_cnt: i32) -> i32 {
    todo!();
}

unsafe extern "C" fn btdm_sleep_enter_phase1(_lpcycles: i32) {
    todo!();
}

unsafe extern "C" fn btdm_sleep_enter_phase2() {
    todo!();
}

unsafe extern "C" fn btdm_sleep_exit_phase1() {
    todo!();
}

unsafe extern "C" fn btdm_sleep_exit_phase2() {
    todo!();
}

unsafe extern "C" fn btdm_sleep_exit_phase3() {
    todo!();
}

unsafe extern "C" fn coex_schm_status_bit_set(_typ: i32, _status: i32) {
    trace!("!!! unimplemented coex_schm_status_bit_set");
}

unsafe extern "C" fn coex_schm_status_bit_clear(_typ: i32, _status: i32) {
    trace!("!!! unimplemented coex_schm_status_bit_clear");
}

unsafe extern "C" fn read_efuse_mac(mac: *const ()) -> i32 {
    crate::wifi::read_mac(mac as *mut _, 2)
}

pub fn ble_init() {
    unsafe {
        BT_INTERNAL_QUEUE = Some(SimpleQueue::new());
        BT_RECEIVE_QUEUE = Some(SimpleQueue::new());

        *(HCI_OUT_COLLECTOR.as_mut_ptr()) = HciOutCollector::new();

        // turn on logging
        #[cfg(feature = "wifi_logs")]
        {
            extern "C" {
                static mut g_bt_plf_log_level: u32;
            }

            log::info!("g_bt_plf_log_level = {}", g_bt_plf_log_level);
            g_bt_plf_log_level = 10;
        }

        // esp32_bt_controller_init

        let mut cfg = ble_os_adapter_chip_specific::create_ble_config();

        let res = btdm_osi_funcs_register(&G_OSI_FUNCS as *const _ as *const ());
        if res != 0 {
            panic!("btdm_osi_funcs_register returned {}", res);
        }

        let version = btdm_controller_get_compile_version();
        let version_str = StrBuf::from(version);
        log::debug!("BT controller compile version {}", version_str.as_str_ref());

        ble_os_adapter_chip_specific::btdm_controller_mem_init();

        ble_os_adapter_chip_specific::bt_periph_module_enable();

        ble_os_adapter_chip_specific::disable_sleep_mode();

        #[cfg(feature = "esp32c3")]
        let res = btdm_controller_init(&mut cfg as *mut esp_bt_controller_config_t);

        #[cfg(feature = "esp32")]
        let res = btdm_controller_init(
            (1 << 3) | (1 << 4),
            &mut cfg as *mut esp_bt_controller_config_t,
        ); // see btdm_config_mask_load for mask

        if res != 0 {
            panic!("btdm_controller_init returned {}", res);
        }

        log::debug!("The btdm_controller_init was initialized");

        #[cfg(feature = "esp32")]
        {
            extern "C" {
                fn coex_ble_adv_priority_high_set(high: bool);
            }

            coex_ble_adv_priority_high_set(false);
        }

        // esp32_bt_controller_enable

        // modifyreg32(SYSTEM_WIFI_CLK_EN_REG, 0, UINT32_MAX);
        // bt_phy_enable();
        crate::wifi::os_adapter::phy_enable();

        #[cfg(feature = "esp32")]
        {
            extern "C" {
                fn btdm_rf_bb_init_phase2();
            }

            btdm_rf_bb_init_phase2();
            coex_bt_high_prio();
        }

        btdm_controller_enable(esp_bt_mode_t_ESP_BT_MODE_BLE); // fails with assertion

        API_vhci_host_register_callback(&VHCI_HOST_CALLBACK);

        critical_section::with(|_| {
            BLE_INITIALIZED = true;
        });
    }
}

pub fn send_hci(data: &[u8]) {
    let hci_out = unsafe { &mut *HCI_OUT_COLLECTOR.as_mut_ptr() };
    hci_out.push(data);

    if hci_out.is_ready() {
        let packet = hci_out.packet();

        unsafe {
            loop {
                let can_send = API_vhci_host_check_send_available();

                if !can_send {
                    log::trace!("can_send is false");
                    continue;
                }

                API_vhci_host_send_packet(packet.as_ptr() as *const u8, packet.len() as u16);
                log::trace!("sent vhci host packet");

                break;
            }
        }

        hci_out.reset();
    }
}

static mut BLE_HCI_READ_DATA: [u8; 256] = [0u8; 256];
static mut BLE_HCI_READ_DATA_INDEX: usize = 0;
static mut BLE_HCI_READ_DATA_LEN: usize = 0;

pub fn read_hci(data: &mut [u8]) -> usize {
    unsafe {
        if BLE_HCI_READ_DATA_LEN == 0 {
            let dequeued = BT_RECEIVE_QUEUE.as_mut().unwrap().dequeue();
            match dequeued {
                Some(packet) => {
                    for i in 0..(packet.len as usize + 0/*1*/) {
                        BLE_HCI_READ_DATA[i] = packet.data[i];
                    }

                    BLE_HCI_READ_DATA_LEN = packet.len as usize + 0 /*1*/;
                    BLE_HCI_READ_DATA_INDEX = 0;
                }
                None => (),
            };
        }

        if BLE_HCI_READ_DATA_LEN > 0 {
            data[0] = BLE_HCI_READ_DATA[BLE_HCI_READ_DATA_INDEX];
            BLE_HCI_READ_DATA_INDEX += 1;

            if BLE_HCI_READ_DATA_INDEX >= BLE_HCI_READ_DATA_LEN {
                BLE_HCI_READ_DATA_LEN = 0;
                BLE_HCI_READ_DATA_INDEX = 0;
            }
            return 1;
        }
    }

    0
}

pub struct HciPipe {
    wbuffer: [u8; 256],
    rbuffer: [u8; 256],
    w_write_idx: usize,
    w_read_idx: usize,
    r_write_idx: usize,
    r_read_idx: usize,
}

impl HciPipe {
    pub fn new() -> HciPipe {
        HciPipe {
            wbuffer: [0u8; 256],
            rbuffer: [0u8; 256],
            w_write_idx: 0,
            w_read_idx: 0,
            r_write_idx: 0,
            r_read_idx: 0,
        }
    }

    pub fn controller_read(&mut self) -> Option<u8> {
        critical_section::with(|_| {
            if self.r_write_idx == self.r_read_idx {
                None
            } else {
                let r = self.rbuffer[self.r_read_idx];
                self.r_read_idx += 1;
                if self.r_read_idx >= self.rbuffer.len() {
                    self.r_read_idx = 0;
                }
                Some(r)
            }
        })
    }

    pub fn controller_write(&mut self, v: u8) {
        critical_section::with(|_| {
            self.wbuffer[self.w_write_idx] = v;
            self.w_write_idx += 1;
            if self.w_write_idx >= self.wbuffer.len() {
                self.w_write_idx = 0;
            }

            if self.w_write_idx == self.w_read_idx {
                panic!("Buffer overflow in controller_write");
            }
        })
    }

    pub fn host_read(&mut self) -> Option<u8> {
        critical_section::with(|_| {
            if self.w_write_idx == self.w_read_idx {
                None
            } else {
                let r = self.wbuffer[self.w_read_idx];
                self.w_read_idx += 1;
                if self.w_read_idx >= self.wbuffer.len() {
                    self.w_read_idx = 0;
                }
                Some(r)
            }
        })
    }

    pub fn host_peek(&mut self, offset: usize) -> Option<u8> {
        critical_section::with(|_| {
            if self.w_write_idx == self.w_read_idx {
                None
            } else {
                let index = (self.w_read_idx + offset) % self.wbuffer.len();

                // ???
                if index > self.w_write_idx {
                    None
                } else {
                    Some(self.wbuffer[index])
                }
            }
        })
    }

    pub fn host_write(&mut self, v: u8) {
        critical_section::with(|_| {
            self.rbuffer[self.r_write_idx] = v;
            self.r_write_idx += 1;
            if self.r_write_idx >= self.rbuffer.len() {
                self.r_write_idx = 0;
            }

            if self.r_write_idx == self.r_read_idx {
                panic!("Buffer overflow in host_write");
            }
        })
    }
}

static mut HCI_OUT_COLLECTOR: MaybeUninit<HciOutCollector> = MaybeUninit::uninit();

#[derive(PartialEq, Debug)]
enum HciOutType {
    Unknown,
    Acl,
    Command,
}

struct HciOutCollector {
    data: [u8; 256],
    index: usize,
    ready: bool,
    kind: HciOutType,
}

impl HciOutCollector {
    fn new() -> HciOutCollector {
        HciOutCollector {
            data: [0u8; 256],
            index: 0,
            ready: false,
            kind: HciOutType::Unknown,
        }
    }

    fn is_ready(&self) -> bool {
        self.ready
    }

    fn push(&mut self, data: &[u8]) {
        self.data[self.index..(self.index + data.len())].copy_from_slice(data);
        self.index += data.len();

        if self.kind == HciOutType::Unknown {
            self.kind = match self.data[0] {
                1 => HciOutType::Command,
                2 => HciOutType::Acl,
                _ => HciOutType::Unknown,
            };
        }

        if !self.ready {
            if self.kind == HciOutType::Command && self.index >= 4 {
                if self.index == self.data[3] as usize + 4 {
                    self.ready = true;
                }
            } else if self.kind == HciOutType::Acl && self.index >= 5 {
                if self.index == (self.data[3] as usize) + ((self.data[4] as usize) << 8) + 5 {
                    self.ready = true;
                }
            }
        }
    }

    fn reset(&mut self) {
        self.index = 0;
        self.ready = false;
        self.kind = HciOutType::Unknown;
    }

    fn packet(&self) -> &[u8] {
        &self.data[0..(self.index as usize)]
    }
}
