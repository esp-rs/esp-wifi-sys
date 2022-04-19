#![no_std]
#![no_main]

use embedded_svc::wifi::{
    ClientConfiguration, ClientConnectionStatus, ClientIpStatus, ClientStatus, Configuration,
    Status, Wifi,
};
use esp32_hal::{pac::Peripherals, RtcCntl};
use esp_wifi::println;
use esp_wifi::wifi::initialize;
use esp_wifi::wifi::utils::create_network_stack;
use esp_wifi::{create_network_stack_storage, network_stack_storage, Uart};
use xtensa_lx_rt::entry;

use embedded_nal::SocketAddrV4;
use embedded_nal::TcpClientStack;

extern crate alloc;

const SSID: &str = env!("SSID");
const PASSWORD: &str = env!("PASSWORD");

#[entry]
fn main() -> ! {
    let peripherals = Peripherals::take().unwrap();

    let mut rtc_cntl = RtcCntl::new(peripherals.RTC_CNTL);

    // Disable MWDT and RWDT (Watchdog) flash boot protection
    rtc_cntl.set_wdt_global_enable(false);

    let mut storage = create_network_stack_storage!(3, 8, 1);
    let network_stack = create_network_stack(network_stack_storage!(storage));

    let mut wifi_interface = esp_wifi::wifi_interface::Wifi::new(network_stack);

    init_logger();

    initialize(peripherals.TIMG1, peripherals.RNG).unwrap();

    println!("{:?}", wifi_interface.get_status());

    println!("Start Wifi Scan");
    let res = wifi_interface.scan();
    if let Ok(res) = res {
        for ap in res {
            println!("{:?}", ap);
        }
    }

    println!("Call wifi_connect");
    let client_config = Configuration::Client(ClientConfiguration {
        ssid: SSID.into(),
        password: PASSWORD.into(),
        ..Default::default()
    });
    let res = wifi_interface.set_configuration(&client_config);
    println!("wifi_connect returned {:?}", res);

    println!("{:?}", wifi_interface.get_capabilities());
    println!("{:?}", wifi_interface.get_status());

    // wait to get connected
    loop {
        if let Status(ClientStatus::Started(_), _) = wifi_interface.get_status() {
            break;
        }
    }
    println!("{:?}", wifi_interface.get_status());

    println!("Start busy loop on main");

    let mut stage = 0;
    let mut socket = None;
    let mut idx = 0;
    let mut buffer = [0u8; 8000];
    let mut waiter = 50000;

    loop {
        wifi_interface.network_stack().poll().ok();

        if let Status(
            ClientStatus::Started(ClientConnectionStatus::Connected(ClientIpStatus::Done(config))),
            _,
        ) = wifi_interface.get_status()
        {
            match stage {
                0 => {
                    println!("My IP config is {:?}", config);
                    println!("Lets connect");
                    let mut sock = wifi_interface.network_stack().socket().unwrap();
                    let addr =
                        SocketAddrV4::new(embedded_nal::Ipv4Addr::new(142, 250, 185, 115), 80);
                    wifi_interface
                        .network_stack()
                        .connect(&mut sock, addr.into())
                        .unwrap();

                    socket = Some(sock);
                    stage = 1;
                    println!("Lets send");
                }
                1 => {
                    if wifi_interface
                        .network_stack()
                        .send(
                            &mut socket.unwrap(),
                            &b"GET / HTTP/1.0\r\nHost: www.mobile-j.de\r\n\r\n"[..],
                        )
                        .is_ok()
                    {
                        stage = 2;
                        println!("Lets receive");
                    }
                }
                2 => {
                    if let Ok(s) = wifi_interface
                        .network_stack()
                        .receive(&mut socket.unwrap(), &mut buffer[idx..])
                    {
                        if s > 0 {
                            idx += s;
                        }
                    } else {
                        stage = 3;

                        for c in &buffer[..idx] {
                            esp_wifi::print!("{}", *c as char);
                        }
                        println!("");
                    }
                }
                3 => {
                    println!("Close");
                    wifi_interface.network_stack().close(socket.unwrap()).ok();
                    stage = 4;
                }
                4 => {
                    waiter -= 1;
                    if waiter == 0 {
                        idx = 0;
                        waiter = 50000;
                        stage = 0;
                    }
                }
                _ => (),
            }
        }
    }
}

#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    println!("\n\n*** {:?}", info);
    loop {}
}

#[no_mangle]
unsafe extern "C" fn __exception(
    cause: xtensa_lx_rt::exception::ExceptionCause,
    context: &xtensa_lx_rt::exception::Context,
) {
    println!("\n\n*** {:?} {:x?}", cause, context);
    loop {}
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
        println!("{} - {}", record.level(), record.args());
    }

    fn flush(&self) {}
}
