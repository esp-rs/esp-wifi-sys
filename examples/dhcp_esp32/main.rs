#![no_std]
#![no_main]

use embedded_svc::wifi::{
    ClientConfiguration, ClientConnectionStatus, ClientIpStatus, ClientStatus, Configuration,
    Status, Wifi,
};
use esp32_hal::{pac::Peripherals, RtcCntl};
use esp_wifi::println;
use esp_wifi::wifi::initialize;
use esp_wifi::wifi::utils::create_network_interface;
use esp_wifi::wifi_interface::timestamp;
use esp_wifi::{create_network_stack_storage, network_stack_storage, Uart};
use smoltcp::iface::SocketHandle;
use smoltcp::socket::{Socket, TcpSocket};
use xtensa_lx_rt::entry;

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
    let ethernet = create_network_interface(network_stack_storage!(storage));
    let mut wifi_interface = esp_wifi::wifi_interface::Wifi::new(ethernet);

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
    let mut idx = 0;
    let mut buffer = [0u8; 8000];
    let mut waiter = 50000;

    let mut http_socket_handle: Option<SocketHandle> = None;

    for (handle, socket) in wifi_interface.network_interface().sockets_mut() {
        match socket {
            Socket::Tcp(_) => http_socket_handle = Some(handle),
            _ => {}
        }
    }

    loop {
        wifi_interface.poll_dhcp().ok();
        wifi_interface.network_interface().poll(timestamp()).ok();

        if let Status(
            ClientStatus::Started(ClientConnectionStatus::Connected(ClientIpStatus::Done(config))),
            _,
        ) = wifi_interface.get_status()
        {
            match stage {
                0 => {
                    println!("My IP config is {:?}", config);
                    println!("Lets connect");
                    let (socket, cx) = wifi_interface
                        .network_interface()
                        .get_socket_and_context::<TcpSocket>(http_socket_handle.unwrap());

                    let address = smoltcp::wire::Ipv4Address::new(142, 250, 185, 115);
                    let remote_endpoint = (address, 80);
                    socket.connect(cx, remote_endpoint, 41000).unwrap();

                    stage = 1;
                    println!("Lets send");
                }
                1 => {
                    let socket = wifi_interface
                        .network_interface()
                        .get_socket::<TcpSocket>(http_socket_handle.unwrap());

                    if socket
                        .send_slice(&b"GET / HTTP/1.0\r\nHost: www.mobile-j.de\r\n\r\n"[..])
                        .is_ok()
                    {
                        stage = 2;
                        println!("Lets receive");
                    }
                }
                2 => {
                    let socket = wifi_interface
                        .network_interface()
                        .get_socket::<TcpSocket>(http_socket_handle.unwrap());

                    if let Ok(s) = socket.recv_slice(&mut buffer[idx..]) {
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
                    let socket = wifi_interface
                        .network_interface()
                        .get_socket::<TcpSocket>(http_socket_handle.unwrap());

                    socket.close();
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
