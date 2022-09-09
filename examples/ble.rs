#![no_std]
#![no_main]
#![feature(c_variadic)]
#![feature(const_mut_refs)]

#[cfg(feature = "esp32")]
use esp32_hal as hal;
#[cfg(feature = "esp32c3")]
use esp32c3_hal as hal;

use ble_hci::{
    ad_structure::{
        create_advertising_data, AdStructure, BR_EDR_NOT_SUPPORTED, LE_GENERAL_DISCOVERABLE,
    },
    att::Uuid,
    attribute_server::{AttributeServer, Service, WorkResult, ATT_READABLE, ATT_WRITEABLE},
    Ble, Data, HciConnector,
};
use esp_backtrace as _;
use esp_println::{logger::init_logger, println};
use esp_wifi::{ble::controller::BleConnector, initialize};
use hal::{
    clock::{ClockControl, CpuClock},
    pac::Peripherals,
    prelude::*,
    Rtc,
};

#[cfg(feature = "esp32c3")]
use hal::system::SystemExt;

#[cfg(feature = "esp32c3")]
use riscv_rt::entry;
#[cfg(feature = "esp32")]
use xtensa_lx_rt::entry;

extern crate alloc;

#[entry]
fn main() -> ! {
    init_logger(log::LevelFilter::Info);
    esp_wifi::init_heap();

    let peripherals = Peripherals::take().unwrap();

    #[cfg(not(feature = "esp32"))]
    let system = peripherals.SYSTEM.split();
    #[cfg(feature = "esp32")]
    let system = peripherals.DPORT.split();

    #[cfg(feature = "esp32c3")]
    let clocks = ClockControl::configure(system.clock_control, CpuClock::Clock160MHz).freeze();
    #[cfg(feature = "esp32")]
    let clocks = ClockControl::configure(system.clock_control, CpuClock::Clock240MHz).freeze();

    let mut rtc = Rtc::new(peripherals.RTC_CNTL);

    // Disable watchdog timers
    #[cfg(not(feature = "esp32"))]
    rtc.swd.disable();

    rtc.rwdt.disable();

    #[cfg(feature = "esp32c3")]
    {
        use hal::systimer::SystemTimer;
        let syst = SystemTimer::new(peripherals.SYSTIMER);
        initialize(syst.alarm0, peripherals.RNG, &clocks).unwrap();
    }
    #[cfg(feature = "esp32")]
    {
        use hal::timer::TimerGroup;
        let timg1 = TimerGroup::new(peripherals.TIMG1, &clocks);
        initialize(timg1.timer0, peripherals.RNG, &clocks).unwrap();
    }

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
                #[cfg(feature = "esp32c3")]
                AdStructure::CompleteLocalName("ESP32-C3 BLE"),
                #[cfg(feature = "esp32")]
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
