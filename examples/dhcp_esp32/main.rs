#![no_std]
#![no_main]
#![feature(alloc_error_handler)]

use core::panic::PanicInfo;

use embedded_svc::wifi::{
    ClientConfiguration, ClientConnectionStatus, ClientIpStatus, ClientStatus, Configuration,
    Status, Wifi,
};
use esp32_hal::{clock::ClockControl, pac::Peripherals, prelude::*, RtcCntl};
use esp_println::{print, println};
use esp_wifi::{
    create_network_stack_storage, network_stack_storage,
    wifi::{initialize, utils::create_network_interface},
    wifi_interface::timestamp,
};
use smoltcp::{
    iface::SocketHandle,
    socket::{Socket, TcpSocket},
};
use xtensa_lx_rt::entry;

extern crate alloc;

const SSID: &str = env!("SSID");
const PASSWORD: &str = env!("PASSWORD");

#[alloc_error_handler]
fn oom(_: core::alloc::Layout) -> ! {
    loop {}
}

// TODO why can't we just use esp-backtrace here?
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}

#[entry]
fn main() -> ! {
    init_logger();
    esp_wifi::init_heap();

    let peripherals = Peripherals::take().unwrap();
    let system = peripherals.DPORT.split();
    let clocks = ClockControl::boot_defaults(system.clock_control).freeze();

    let mut rtc_cntl = RtcCntl::new(peripherals.RTC_CNTL);

    // Disable MWDT and RWDT (Watchdog) flash boot protection
    rtc_cntl.set_wdt_global_enable(false);

    let mut storage = create_network_stack_storage!(3, 8, 1);
    let ethernet = create_network_interface(network_stack_storage!(storage));
    let mut wifi_interface = esp_wifi::wifi_interface::Wifi::new(ethernet);

    initialize(peripherals.TIMG1, peripherals.RNG, &clocks).unwrap();

    println!("{:?}", wifi_interface.get_status());

    println!("Start Wifi Scan");
    let res = wifi_interface.scan_n::<10>();
    if let Ok((res, _count)) = res {
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
                            print!("{}", *c as char);
                        }
                        println!("");
                    }
                }
                3 => {
                    println!("Close");
                    let socket = wifi_interface
                        .network_interface()
                        .get_socket::<TcpSocket>(http_socket_handle.unwrap());

                    socket.abort();
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
