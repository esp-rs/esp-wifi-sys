#![no_std]
#![no_main]
#![feature(c_variadic)]
#![feature(const_mut_refs)]

use embedded_svc::wifi::{
    AccessPointInfo, ClientConfiguration, ClientConnectionStatus, ClientIpStatus, ClientStatus,
    Configuration, Status, Wifi,
};
use esp32c3_hal::clock::{ClockControl, CpuClock};
use esp32c3_hal::system::SystemExt;
use esp32c3_hal::{pac::Peripherals, RtcCntl};
use esp_backtrace as _;
use esp_println::{print, println};
use esp_wifi::wifi::{initialize, WifiDevice};
use esp_wifi::wifi::utils::create_network_interface;
use esp_wifi::wifi_interface::{timestamp, WifiError};
use esp_wifi::{create_network_stack_storage, network_stack_storage};
use riscv_rt::entry;
use smoltcp::{
    iface::SocketHandle,
    socket::tcp,
};

extern crate alloc;

const SSID: &str = env!("SSID");
const PASSWORD: &str = env!("PASSWORD");

#[entry]
fn main() -> ! {
    init_logger();
    esp_wifi::init_heap();

    let mut peripherals = Peripherals::take().unwrap();
    let system = peripherals.SYSTEM.split();
    let clocks = ClockControl::configure(system.clock_control, CpuClock::Clock160MHz).freeze();

    let mut rtc_cntl = RtcCntl::new(peripherals.RTC_CNTL);

    // Disable watchdog timers
    rtc_cntl.set_super_wdt_enable(false);
    rtc_cntl.set_wdt_global_enable(false);

    let mut wifi_device = WifiDevice::new();

    let mut storage = create_network_stack_storage!(4, 8, 1);
    let (ethernet, mut sockets) = create_network_interface(&mut wifi_device, network_stack_storage!(storage));

    // // TODO get this from DHCP instead
    // let servers = &[
    //     Ipv4Address::new(8, 8, 4, 4).into(),
    //     Ipv4Address::new(8, 8, 8, 8).into(),
    // ];
    // static mut DNS_SOCKET_STORAGE: [u8; 2500] = [0; 2500];
    // let dns_socket = smoltcp::socket::dns::Socket::new(servers, unsafe { &mut DNS_SOCKET_STORAGE[..] });
    // ethernet.add_socket(dns_socket);

    let mut dhcp_socket_handle: Option<SocketHandle> = None;

    for (handle, socket) in sockets.iter() {
        match socket {
            smoltcp::socket::Socket::Dhcpv4(_) => dhcp_socket_handle = Some(handle),
            _ => {}
        }
    }

    let mut wifi_interface = esp_wifi::wifi_interface::Wifi::new(ethernet, dhcp_socket_handle);

    initialize(&mut peripherals.SYSTIMER, peripherals.RNG, &clocks).unwrap();

    println!("{:?}", wifi_interface.get_status());

    println!("Start Wifi Scan");
    let res: Result<(heapless::Vec<AccessPointInfo, 10>, usize), WifiError> =
        wifi_interface.scan_n();
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

    for (handle, socket) in sockets.iter() {
        match socket {
            smoltcp::socket::Socket::Tcp(_) => http_socket_handle = Some(handle),
            _ => {}
        }
    }

    loop {
        wifi_interface.poll_dhcp(&mut sockets).ok();
        wifi_interface.network_interface().poll(timestamp(), &mut wifi_device, &mut sockets).ok();
        // wifi_interface.network_interface().poll(timestamp(), &mut wifi_device, sockets).ok();

        if let Status(
            ClientStatus::Started(ClientConnectionStatus::Connected(ClientIpStatus::Done(config))),
            _,
        ) = wifi_interface.get_status()
        {
            match stage {
                0 => {
                    println!("My IP config is {:?}", config);
                    
                }
                1 => {
                    println!("Lets connect");
                    let socket= sockets
                        .get_mut::<tcp::Socket>(http_socket_handle.unwrap());

                    let address = smoltcp::wire::Ipv4Address::new(142, 250, 185, 115);
                    let remote_endpoint = (address, 80);
                    socket.connect(wifi_interface.network_interface().context(), remote_endpoint, 41000).unwrap();

                    stage = 2;
                    println!("Lets send");
                }
                2 => {
                    let socket = sockets
                        .get_mut::<tcp::Socket>(http_socket_handle.unwrap());

                    if socket
                        .send_slice(&b"GET / HTTP/1.0\r\nHost: www.mobile-j.de\r\n\r\n"[..])
                        .is_ok()
                    {
                        stage = 3;
                        println!("Lets receive");
                    }
                }
                3 => {
                    let socket = sockets
                        .get_mut::<tcp::Socket>(http_socket_handle.unwrap());

                    if let Ok(s) = socket.recv_slice(&mut buffer[idx..]) {
                        if s > 0 {
                            idx += s;
                        }
                    } else {
                        stage = 4;

                        for c in &buffer[..idx] {
                            print!("{}", *c as char);
                        }
                        println!("");
                    }
                }
                4 => {
                    println!("Close");
                    let socket = sockets
                        .get_mut::<tcp::Socket>(http_socket_handle.unwrap());

                    socket.abort();
                    stage = 5;
                }
                5 => {
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
