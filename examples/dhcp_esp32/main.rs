#![no_std]
#![no_main]

use esp32_hal::{pac::Peripherals, RtcCntl};
use esp_wifi::wifi::{get_sta_mac, init_rng};
use esp_wifi::Uart;
use esp_wifi::{
    binary, compat, println,
    tasks::init_tasks,
    timer::{get_systimer_count, setup_timer_isr},
    wifi::{self, init_buffer, wifi_connect, wifi_init, wifi_start, WifiDevice},
};
use xtensa_lx_rt::entry;

use smoltcp::iface::{Interface, SocketStorage};
use smoltcp::phy::Device;
use smoltcp::socket::Dhcpv4Event;
use smoltcp::{
    iface::{NeighborCache, Routes},
    socket::{Dhcpv4Socket, TcpSocket, TcpSocketBuffer},
    time::Instant,
    wire::{EthernetAddress, IpCidr, Ipv4Address, Ipv4Cidr},
};

const SSID: &str = env!("SSID");
const PASSWORD: &str = env!("PASSWORD");

#[entry]
fn main() -> ! {
    let peripherals = Peripherals::take().unwrap();

    init_rng(peripherals.RNG);

    let mut rtc_cntl = RtcCntl::new(peripherals.RTC_CNTL);

    // Disable MWDT and RWDT (Watchdog) flash boot protection
    rtc_cntl.set_wdt_global_enable(false);

    // TODO this breaks things currently in phy_enable
    //init_clocks();
    //println!("init clocks done");

    init_tasks();
    setup_timer_isr(peripherals.TIMG1);

    println!("About to make the first call ...");
    println!("Start!");

    let res = wifi_init();
    println!("\n\n\nesp_wifi_init_internal returned {}", res);

    println!("\n\n\nCall wifi_start");
    let res = wifi_start();
    println!("\n\n\nwifi_start returned {}", res);

    println!("Call wifi_start_scan");
    let res = wifi::wifi_start_scan();
    println!("wifi_start_scan returned {}", res);
    print_scan_result();
    println!("\n\n\n\n");

    init_buffer();

    let mut mac = [0u8; 6];
    get_sta_mac(&mut mac);
    println!("MAC address is {:x?}", mac);
    let hw_address = EthernetAddress::from_bytes(&mac);

    let mut socket_set_entries: [SocketStorage; 2] = Default::default();
    let mut neighbor_cache_storage = [None; 8];
    let neighbor_cache = NeighborCache::new(&mut neighbor_cache_storage[..]);

    let device = WifiDevice::new();

    let ip_addr = IpCidr::new(Ipv4Address::UNSPECIFIED.into(), 0);
    let mut ip_addrs = [ip_addr];

    let mut routes_storage = [None; 1];
    let routes = Routes::new(&mut routes_storage[..]);

    let mut ethernet = smoltcp::iface::InterfaceBuilder::new(device, &mut socket_set_entries[..])
        .hardware_addr(smoltcp::wire::HardwareAddress::Ethernet(hw_address))
        .neighbor_cache(neighbor_cache)
        .ip_addrs(&mut ip_addrs[..])
        .routes(routes)
        .finalize();

    println!("Call wifi_connect");
    let res = wifi_connect(SSID, PASSWORD);
    println!("wifi_connect returned {}", res);

    loop {
        if wifi::is_connected() {
            break;
        }
    }

    println!("Start busy loop on main");
    let greet_socket = {
        static mut TCP_SERVER_RX_DATA: [u8; 32] = [0; 32];
        static mut TCP_SERVER_TX_DATA: [u8; 32] = [0; 32];

        let tcp_rx_buffer = unsafe { TcpSocketBuffer::new(&mut TCP_SERVER_RX_DATA[..]) };
        let tcp_tx_buffer = unsafe { TcpSocketBuffer::new(&mut TCP_SERVER_TX_DATA[..]) };

        TcpSocket::new(tcp_rx_buffer, tcp_tx_buffer)
    };
    let greet_handle = ethernet.add_socket(greet_socket);

    let dhcp_socket = Dhcpv4Socket::new();
    let dhcp_handle = ethernet.add_socket(dhcp_socket);

    loop {
        let timestamp = timestamp();
        critical_section::with(|_| {
            ethernet.poll(timestamp).ok();
        });

        let event = ethernet.get_socket::<Dhcpv4Socket>(dhcp_handle).poll();
        match event {
            None => {}
            Some(Dhcpv4Event::Configured(config)) => {
                println!("IP address:      {}", config.address);
                set_ipv4_addr(&mut ethernet, config.address);

                if let Some(router) = config.router {
                    println!("Default gateway: {}", router);
                    ethernet
                        .routes_mut()
                        .add_default_ipv4_route(router)
                        .unwrap();
                } else {
                    println!("Default gateway: None");
                    ethernet.routes_mut().remove_default_ipv4_route();
                }

                for (i, s) in config.dns_servers.iter().enumerate() {
                    if let Some(s) = s {
                        println!("DNS server {}:    {}", i, s);
                    }
                }
            }
            Some(Dhcpv4Event::Deconfigured) => {
                println!("DHCP lost config!");
                set_ipv4_addr(&mut ethernet, Ipv4Cidr::new(Ipv4Address::UNSPECIFIED, 0));
                ethernet.routes_mut().remove_default_ipv4_route();
            }
        }

        // Control the "greeting" socket (:4321)
        {
            let socket = ethernet.get_socket::<TcpSocket>(greet_handle);
            if !socket.is_open() {
                println!(
                    "Listening to port 4321 for greeting, \
                        please connect to the port"
                );
                socket.listen(4321).unwrap();
            }

            if socket.can_send() {
                println!("Send and close.");
                socket.send_slice(&b"Hello World"[..]).ok();
                socket.close();
            }
        }
    }
}

fn timestamp() -> Instant {
    Instant::from_millis((get_systimer_count() / 40_000) as i64)
}

fn set_ipv4_addr<DeviceT>(iface: &mut Interface<'_, DeviceT>, cidr: Ipv4Cidr)
where
    DeviceT: for<'d> Device<'d>,
{
    iface.update_ip_addrs(|addrs| {
        let dest = addrs.iter_mut().next().unwrap();
        *dest = IpCidr::Ipv4(cidr);
    });
}

#[allow(dead_code)]
fn print_scan_result() {
    unsafe {
        let mut bss_total: u16 = 0;
        binary::include::esp_wifi_scan_get_ap_num(&mut bss_total);
        crate::println!("Found {} APs.", bss_total);
        if bss_total > 10 {
            bss_total = 10;
        }

        crate::println!("...");
        let mut records = [binary::include::wifi_ap_record_t {
            bssid: [0u8; 6],
            ssid: [0u8; 33],
            primary: 0u8,
            second: 0u32,
            rssi: 0i8,
            authmode: 0u32,
            pairwise_cipher: 0u32,
            group_cipher: 0u32,
            ant: 0u32,
            _bitfield_align_1: [0u32; 0],
            _bitfield_1: binary::include::__BindgenBitfieldUnit::new([0u8; 4usize]),
            country: binary::include::wifi_country_t {
                cc: [0; 3],
                schan: 0u8,
                nchan: 0u8,
                max_tx_power: 0i8,
                policy: 0u32,
            },
        }; 10];

        crate::println!("calling esp_wifi_scan_get_ap_records");
        binary::include::esp_wifi_scan_get_ap_records(
            &mut bss_total,
            &mut records as *mut binary::include::wifi_ap_record_t,
        );

        crate::println!("printing {} records", bss_total);
        for i in 0..bss_total {
            let record = records[i as usize];
            let ssid = compat::common::StrBuf::from(&record.ssid as *const u8);
            crate::println!(
                "{} {} {:x?} {}",
                ssid.as_str_ref(),
                record.rssi,
                record.bssid,
                record.primary
            );
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
