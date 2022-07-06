#![no_std]
#![no_main]
#![feature(alloc_error_handler)]
#![feature(c_variadic)]
#![feature(const_mut_refs)]

use ble_hci::{
    ad_structure::{
        create_advertising_data,
        AdStructure,
        BR_EDR_NOT_SUPPORTED,
        LE_GENERAL_DISCOVERABLE,
    },
    att::Uuid,
    attribute_server::{AttributeServer, Service, WorkResult, ATT_READABLE, ATT_WRITEABLE},
    Ble,
    Data,
    HciConnector,
};
use esp32_hal::{clock::ClockControl, pac::Peripherals, prelude::*, RtcCntl};
// use esp_backtrace as _;
use esp_println::println;
use esp_wifi::{
    ble::{ble_init, controller::BleConnector},
    wifi::initialize_ble,
};
use xtensa_lx_rt::entry;

extern crate alloc;

#[global_allocator]
static ALLOCATOR: esp_alloc::EspHeap = esp_alloc::EspHeap::empty();

fn init_heap() {
    use core::mem::MaybeUninit;

    const HEAP_SIZE: usize = 4 * 1024;
    static mut HEAP: [MaybeUninit<u8>; HEAP_SIZE] = [MaybeUninit::uninit(); HEAP_SIZE];

    unsafe {
        ALLOCATOR.init(HEAP.as_ptr() as usize, HEAP_SIZE);
    }
}

#[alloc_error_handler]
fn oom(_: core::alloc::Layout) -> ! {
    loop {}
}

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {}
}

#[entry]
fn main() -> ! {
    esp_wifi::init_heap();
    init_heap();

    let peripherals = Peripherals::take().unwrap();
    let system = peripherals.DPORT.split();
    let clocks = ClockControl::boot_defaults(system.clock_control).freeze();

    let mut rtc_cntl = RtcCntl::new(peripherals.RTC_CNTL);

    // Disable MWDT and RWDT (Watchdog) flash boot protection
    rtc_cntl.set_wdt_global_enable(false);

    init_logger();

    initialize_ble(peripherals.TIMG1, peripherals.RNG, &clocks).unwrap();

    println!("before ble init");
    ble_init();
    println!("after ble init");

    loop {
        let connector = BleConnector {};
        let hci = HciConnector::new(connector, esp_wifi::current_millis);
        let mut ble = Ble::new(&hci);

        println!("{:?}", ble.init());
        println!("{:?}", ble.cmd_set_le_advertising_parameters());
        println!(
            "{:?}",
            ble.cmd_set_le_advertising_data(create_advertising_data(&[
                AdStructure::Flags(LE_GENERAL_DISCOVERABLE | BR_EDR_NOT_SUPPORTED),
                AdStructure::ServiceUuids16(&[Uuid::Uuid16(0x1809)]),
                AdStructure::CompleteLocalName("ESP32 BLE"),
            ]))
        );
        println!("{:?}", ble.cmd_set_le_advertise_enable(true));

        println!("started advertising");

        let mut rf = || Data::new(b"Hello Bare-Metal BLE");
        let mut wf = |data: Data| {
            println!("RECEIVED: {:x?}", data.to_slice());
        };

        let srv1 = Service::new(
            Uuid::Uuid128([
                0xC9, 0x15, 0x15, 0x96, 0x54, 0x56, 0x64, 0xB3, 0x38, 0x45, 0x26, 0x5D, 0xF1, 0x62,
                0x6A, 0xA8,
            ]),
            ATT_READABLE | ATT_WRITEABLE,
            &mut rf,
            &mut wf,
        );

        let mut rf2 = || Data::default();
        let mut wf2 = |_data| {};

        let srv2 = Service::new(
            Uuid::Uuid128([
                0xC8, 0x15, 0x15, 0x96, 0x54, 0x56, 0x64, 0xB3, 0x38, 0x45, 0x26, 0x5D, 0xF1, 0x62,
                0x6A, 0xA8,
            ]),
            ATT_WRITEABLE,
            &mut rf2,
            &mut wf2,
        );

        let services = &mut [srv1, srv2];
        let mut srv = AttributeServer::new(&mut ble, services);

        loop {
            match srv.do_work() {
                Ok(res) => {
                    if let WorkResult::GotDisconnected = res {
                        break;
                    }
                }
                Err(err) => {
                    println!("{:x?}", err);
                }
            }
        }
    }
}

pub fn init_logger() {
    unsafe {
        log::set_logger_racy(&LOGGER).unwrap();
        log::set_max_level(log::LevelFilter::Info);
    }
}

static LOGGER: SimpleLogger = SimpleLogger;
struct SimpleLogger;

impl log::Log for SimpleLogger {
    fn enabled(&self, _metadata: &log::Metadata) -> bool {
        true
    }

    fn log(&self, record: &log::Record) {
        critical_section::with(|_| {
            let task_id = esp_wifi::preempt::preempt::current_task();
            println!("{} {} - {}", task_id, record.level(), record.args());
        });
    }

    fn flush(&self) {}
}
