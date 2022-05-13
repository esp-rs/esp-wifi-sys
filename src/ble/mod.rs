use core::mem::MaybeUninit;
use log::trace;

use crate::compat::common::StrBuf;
use crate::compat::queue::SimpleQueue;

use crate::binary::include::*;
use crate::compat::work_queue::queue_work;
use esp_alloc::memory_fence;

use esp32c3_hal as hal;

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
    fn btdm_controller_rom_data_init() -> i32;
    fn btdm_osi_funcs_register(osi_funcs: *const ()) -> i32;
    fn btdm_controller_get_compile_version() -> *const u8;
    fn btdm_controller_init(config_opts: *const esp_bt_controller_config_t) -> i32;

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

#[repr(C)]
struct osi_funcs_s {
    magic: u32,
    version: u32,
    interrupt_set: ::core::option::Option<unsafe extern "C" fn(i32, i32, i32, i32) -> ()>,
    interrupt_clear: ::core::option::Option<unsafe extern "C" fn(i32, i32) -> ()>,
    interrupt_handler_set:
        ::core::option::Option<unsafe extern "C" fn(i32, extern "C" fn(), *const ()) -> ()>,
    interrupt_disable: ::core::option::Option<unsafe extern "C" fn() -> ()>,
    interrupt_enable: ::core::option::Option<unsafe extern "C" fn() -> ()>,
    task_yield: ::core::option::Option<unsafe extern "C" fn() -> ()>,
    task_yield_from_isr: ::core::option::Option<unsafe extern "C" fn() -> ()>,
    semphr_create: ::core::option::Option<unsafe extern "C" fn(u32, u32) -> *const ()>,
    semphr_delete: ::core::option::Option<unsafe extern "C" fn(*const ()) -> ()>,
    semphr_take_from_isr: ::core::option::Option<unsafe extern "C" fn(*const (), *const ()) -> i32>,
    semphr_give_from_isr: ::core::option::Option<unsafe extern "C" fn(*const (), *const ()) -> i32>,
    semphr_take: ::core::option::Option<unsafe extern "C" fn(*const (), u32) -> i32>,
    semphr_give: ::core::option::Option<unsafe extern "C" fn(*const ()) -> i32>,
    mutex_create: ::core::option::Option<unsafe extern "C" fn() -> *const ()>,
    mutex_delete: ::core::option::Option<unsafe extern "C" fn(*const ()) -> ()>,
    mutex_lock: ::core::option::Option<unsafe extern "C" fn(*const ()) -> i32>,
    mutex_unlock: ::core::option::Option<unsafe extern "C" fn(*const ()) -> i32>,
    queue_create: ::core::option::Option<unsafe extern "C" fn(u32, u32) -> *const ()>,
    queue_delete: ::core::option::Option<unsafe extern "C" fn(*const ()) -> ()>,
    queue_send: ::core::option::Option<unsafe extern "C" fn(*const (), *const (), u32) -> i32>,
    queue_send_from_isr:
        ::core::option::Option<unsafe extern "C" fn(*const (), *const (), *const ()) -> i32>,
    queue_recv: ::core::option::Option<unsafe extern "C" fn(*const (), *const (), u32) -> i32>,
    queue_recv_from_isr:
        ::core::option::Option<unsafe extern "C" fn(*const (), *const (), *const ()) -> i32>,
    task_create: ::core::option::Option<
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
    task_delete: ::core::option::Option<unsafe extern "C" fn(*const ()) -> ()>,
    is_in_isr: ::core::option::Option<unsafe extern "C" fn() -> i32>,
    cause_sw_intr_to_core: ::core::option::Option<unsafe extern "C" fn(i32, i32) -> i32>,
    malloc: ::core::option::Option<unsafe extern "C" fn(u32) -> *const ()>,
    malloc_internal: ::core::option::Option<unsafe extern "C" fn(u32) -> *const ()>,
    free: ::core::option::Option<unsafe extern "C" fn(*const ()) -> ()>,
    read_efuse_mac: ::core::option::Option<unsafe extern "C" fn(*const ()) -> i32>,
    srand: ::core::option::Option<unsafe extern "C" fn(u32) -> ()>,
    rand: ::core::option::Option<unsafe extern "C" fn() -> i32>,
    btdm_lpcycles_2_hus: ::core::option::Option<unsafe extern "C" fn(u32, u32) -> u32>,
    btdm_hus_2_lpcycles: ::core::option::Option<unsafe extern "C" fn(u32) -> u32>,
    btdm_sleep_check_duration: ::core::option::Option<unsafe extern "C" fn(i32) -> i32>,
    btdm_sleep_enter_phase1: ::core::option::Option<unsafe extern "C" fn(i32) -> ()>,
    btdm_sleep_enter_phase2: ::core::option::Option<unsafe extern "C" fn() -> ()>,
    btdm_sleep_exit_phase1: ::core::option::Option<unsafe extern "C" fn() -> ()>,
    btdm_sleep_exit_phase2: ::core::option::Option<unsafe extern "C" fn() -> ()>,
    btdm_sleep_exit_phase3: ::core::option::Option<unsafe extern "C" fn() -> ()>,
    coex_wifi_sleep_set: ::core::option::Option<unsafe extern "C" fn(i32) -> ()>,
    coex_core_ble_conn_dyn_prio_get:
        ::core::option::Option<unsafe extern "C" fn(*mut i32, *mut i32) -> i32>,
    coex_schm_status_bit_set: ::core::option::Option<unsafe extern "C" fn(i32, i32) -> ()>,
    coex_schm_status_bit_clear: ::core::option::Option<unsafe extern "C" fn(i32, i32) -> ()>,
    interrupt_on: ::core::option::Option<unsafe extern "C" fn(i32) -> ()>,
    interrupt_off: ::core::option::Option<unsafe extern "C" fn(i32) -> ()>,
    esp_hw_power_down: ::core::option::Option<unsafe extern "C" fn() -> ()>,
    esp_hw_power_up: ::core::option::Option<unsafe extern "C" fn() -> ()>,
    ets_backup_dma_copy: ::core::option::Option<unsafe extern "C" fn(u32, u32, u32, i32) -> ()>,
}

static G_OSI_FUNCS: osi_funcs_s = osi_funcs_s {
    magic: 0xfadebead,
    version: 0x00010006,
    interrupt_set: Some(interrupt_set),
    interrupt_clear: Some(interrupt_clear),
    interrupt_handler_set: Some(interrupt_handler_set),
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
    coex_wifi_sleep_set: Some(coex_wifi_sleep_set),
    coex_core_ble_conn_dyn_prio_get: Some(coex_core_ble_conn_dyn_prio_get),
    coex_schm_status_bit_set: Some(coex_schm_status_bit_set),
    coex_schm_status_bit_clear: Some(coex_schm_status_bit_clear),
    interrupt_on: Some(interrupt_on),
    interrupt_off: Some(interrupt_off),
    esp_hw_power_down: Some(esp_hw_power_down),
    esp_hw_power_up: Some(esp_hw_power_up),
    ets_backup_dma_copy: Some(ets_backup_dma_copy),
};

pub(crate) static mut BT_INTERRUPT_FUNCTION5: (
    *mut crate::binary::c_types::c_void,
    *mut crate::binary::c_types::c_void,
) = (core::ptr::null_mut(), core::ptr::null_mut());

pub(crate) static mut BT_INTERRUPT_FUNCTION8: (
    *mut crate::binary::c_types::c_void,
    *mut crate::binary::c_types::c_void,
) = (core::ptr::null_mut(), core::ptr::null_mut());

unsafe extern "C" fn interrupt_set(
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

unsafe extern "C" fn interrupt_clear(_interrupt_source: i32, _interrupt_no: i32) {
    todo!();
}

unsafe extern "C" fn interrupt_handler_set(
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
    todo!();
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
        "unimplemented task_create {:p} {:p} {} {} {:p} {} {:p} {}",
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
    todo!();
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

unsafe extern "C" fn btdm_hus_2_lpcycles(_us: u32) -> u32 {
    todo!();
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

unsafe extern "C" fn coex_wifi_sleep_set(sleep: i32) {
    trace!("!!!! unimplemented coex_wifi_sleep_set {}", sleep);
}

unsafe extern "C" fn coex_core_ble_conn_dyn_prio_get(_low: *mut i32, _high: *mut i32) -> i32 {
    todo!();
}

unsafe extern "C" fn coex_schm_status_bit_set(_typ: i32, _status: i32) {
    trace!("!!! unimplemented coex_schm_status_bit_set");
}

unsafe extern "C" fn coex_schm_status_bit_clear(_typ: i32, _status: i32) {
    trace!("!!! unimplemented coex_schm_status_bit_clear");
}

unsafe extern "C" fn interrupt_on(intr_num: i32) {
    trace!("interrupt_on {}", intr_num);

    (*hal::pac::INTERRUPT_CORE0::PTR)
        .cpu_int_enable
        .modify(|r, w| w.bits(r.bits() | 1 << intr_num));
}

unsafe extern "C" fn interrupt_off(_intr_num: i32) {
    todo!();
}

unsafe extern "C" fn esp_hw_power_down() {
    todo!();
}

unsafe extern "C" fn esp_hw_power_up() {
    todo!();
}

unsafe extern "C" fn ets_backup_dma_copy(_reg: u32, _mem_addr: u32, _num: u32, _to_rem: i32) {
    todo!();
}

unsafe extern "C" fn read_efuse_mac(mac: *const ()) -> i32 {
    crate::wifi::read_mac(mac as *mut _, 2)
}

pub fn ble_init() {
    unsafe {
        BT_INTERNAL_QUEUE = Some(SimpleQueue::new());
        BT_RECEIVE_QUEUE = Some(SimpleQueue::new());

        *(HCI_OUT_COLLECTOR.as_mut_ptr()) = HciOutCollector::new();

        let mut cfg = esp_bt_controller_config_t {
            magic: 0x5A5AA5A5,
            version: 0x02104270,
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
        };

        btdm_controller_rom_data_init();

        let res = btdm_osi_funcs_register(&G_OSI_FUNCS as *const _ as *const ());
        if res != 0 {
            panic!("btdm_osi_funcs_register returned {}", res);
        }

        let version = btdm_controller_get_compile_version();
        let version_str = StrBuf::from(version);
        trace!("BT controller compile version {}", version_str.as_str_ref());

        // modifyreg32(SYSTEM_WIFI_CLK_EN_REG, 0, UINT32_MAX);
        // bt_phy_enable();
        crate::wifi::os_adapter::phy_enable();

        let res = btdm_controller_init(&mut cfg as *mut esp_bt_controller_config_t);
        if res != 0 {
            panic!("btdm_controller_init returned {}", res);
        }

        API_vhci_host_register_callback(&VHCI_HOST_CALLBACK);

        btdm_controller_enable(esp_bt_mode_t_ESP_BT_MODE_BLE);

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
                    continue;
                }

                API_vhci_host_send_packet(packet.as_ptr() as *const u8, packet.len() as u16);
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

                    // BLE_HCI_READ_DATA[0] = match packet.packet_type {
                    //     1 /*BtPacketType::BT_EVT*/ => 4,
                    //     3 /*BtPacketType::BT_ACL_IN*/ => 2,
                    //     _ => 4,
                    // };

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
