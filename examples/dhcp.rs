#![no_std]
#![no_main]
#![feature(c_variadic)]
#![feature(const_mut_refs)]

use core::{arch::asm, fmt::Write};

use esp32c3_wifi_rs::wifi::{get_sta_mac, init_clocks, init_rng};
use esp32c3_wifi_rs::Uart;
use esp32c3_wifi_rs::{
    binary, compat, println,
    timer::{get_systimer_count, init_intr11, init_tasks, setup_timer_isr},
    wifi::{
        self, init_buffer, wifi_connect, wifi_init, wifi_set_log_verbose, wifi_start, WifiDevice,
    },
};
use hal::{interrupt::TrapFrame, pac::Peripherals, RtcCntl, Serial};
use riscv_rt::entry;
use smoltcp::{
    dhcp::Dhcpv4Client,
    iface::{NeighborCache, Routes},
    socket::{RawPacketMetadata, RawSocketBuffer, TcpSocket, TcpSocketBuffer},
    time::Instant,
    wire::{EthernetAddress, IpCidr, Ipv4Address, Ipv4Cidr},
};

const SSID: &str = env!("SSID");
const PASSWORD: &str = env!("PASSWORD");

#[entry]
fn main() -> ! {
    let peripherals = Peripherals::take().unwrap();
    setup_timer_isr(&peripherals);
    init_intr11(&peripherals);
    init_rng(peripherals.RNG);

    let mut rtc_cntl = RtcCntl::new(peripherals.RTC_CNTL);
    let mut serial0 = Serial::new(peripherals.UART0).unwrap();

    // Disable watchdog timers
    rtc_cntl.set_super_wdt_enable(false);
    rtc_cntl.set_wdt_enable(false);

    init_tasks();

    writeln!(serial0, "About to make the first call ...").unwrap();
    println!("Start!");

    wifi_set_log_verbose();

    init_clocks();

    let res = wifi_init();
    println!("\n\n\nesp_wifi_init_internal returned {}", res);

    println!("\n\n\nCall wifi_start");
    let res = wifi_start();
    println!("\n\n\nwifi_start returned {}", res);

    // println!("Call wifi_start_scan");
    // let res = wifi::wifi_start_scan();
    // println!("wifi_start_scan returned {}", res);
    print_scan_result();
    println!("\n\n\n\n");

    init_buffer();

    let mut socket_set_entries: [_; 2] = Default::default();
    let mut sockets = smoltcp::socket::SocketSet::new(&mut socket_set_entries[..]);
    let mut neighbor_cache_storage = [None; 8];
    let neighbor_cache = NeighborCache::new(&mut neighbor_cache_storage[..]);

    let hw_address = EthernetAddress::from_bytes(&[0, 0, 0, 0, 0, 0]);
    let device = WifiDevice::new();

    let ip_addr = IpCidr::new(Ipv4Address::UNSPECIFIED.into(), 0);
    let mut ip_addrs = [ip_addr];

    let mut dhcp_rx_buff = [0u8; 900];
    let mut dhcp_rx_metadata_storage = [RawPacketMetadata::EMPTY; 1];
    let dhcp_rx_buffer =
        RawSocketBuffer::new(&mut dhcp_rx_metadata_storage[..], &mut dhcp_rx_buff[..]);
    let mut dhcp_tx_buff = [0u8; 600];
    let mut dhcp_tx_metadata_storage = [RawPacketMetadata::EMPTY; 1];
    let dhcp_tx_buffer =
        RawSocketBuffer::new(&mut dhcp_tx_metadata_storage[..], &mut dhcp_tx_buff[..]);
    let mut dhcp = Dhcpv4Client::new(&mut sockets, dhcp_rx_buffer, dhcp_tx_buffer, timestamp());
    let mut prev_cidr = Ipv4Cidr::new(Ipv4Address::UNSPECIFIED, 0);

    let mut routes_storage = [None; 1];
    let routes = Routes::new(&mut routes_storage[..]);

    let mut ethernet = smoltcp::iface::EthernetInterfaceBuilder::new(device)
        .ethernet_addr(hw_address)
        .neighbor_cache(neighbor_cache)
        .ip_addrs(&mut ip_addrs[..])
        .routes(routes)
        .finalize();

    // need to tell SmolTCP our MAC
    let mut mac = [0u8; 6];
    get_sta_mac(&mut mac);
    println!("MAC address is {:x?}", mac);
    let addr = EthernetAddress::from_bytes(&mac);
    ethernet.set_ethernet_addr(addr);

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
    let greet_handle = sockets.add(greet_socket);

    loop {
        let timestamp = timestamp();
        critical_section::with(|_| {
            ethernet.poll(&mut sockets, timestamp).ok();
        });

        let config = dhcp
            .poll(&mut ethernet, &mut sockets, timestamp)
            .unwrap_or_else(|e| {
                println!("DHCP Error: {:?}", e);
                None
            });

        config.map(|config| {
            println!("DHCP config: {:?}", config);
            if let Some(cidr) = config.address {
                if cidr != prev_cidr {
                    ethernet.update_ip_addrs(|addrs| {
                        addrs.iter_mut().next().map(|addr| {
                            *addr = IpCidr::Ipv4(cidr);
                        });
                    });
                    prev_cidr = cidr;
                    println!("Assigned a new IPv4 address: {}", cidr);
                }
            }

            config.router.map(|router| {
                ethernet
                    .routes_mut()
                    .add_default_ipv4_route(router)
                    .unwrap()
            });
            ethernet.routes_mut().update(|routes_map| {
                routes_map
                    .get(&IpCidr::new(Ipv4Address::UNSPECIFIED.into(), 0))
                    .map(|default_route| {
                        println!("Default gateway: {}", default_route.via_router);
                    });
            });

            if config.dns_servers.iter().any(|s| s.is_some()) {
                println!("DNS servers:");
                for dns_server in config.dns_servers.iter().filter_map(|s| *s) {
                    println!("- {}", dns_server);
                }
            }
        });

        // Control the "greeting" socket (:4321)
        {
            let mut socket = sockets.get::<TcpSocket>(greet_handle);
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
    Instant {
        millis: (get_systimer_count() / 16_000) as i64,
    }
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

#[export_name = "DefaultHandler"]
pub fn default_handler() {
    println!("DefaultHandler called!");
}

#[export_name = "ExceptionHandler"]
fn custom_exception_handler(_trap_frame: &TrapFrame) -> ! {
    let mepc = riscv::register::mepc::read();
    let code = riscv::register::mcause::read().code() & 0xff;
    let mtval = riscv::register::mtval::read();

    let code = match code {
        0 => "Instruction address misaligned",
        1 => "Instruction access fault",
        2 => "Illegal instruction",
        3 => "Breakpoint",
        4 => "Load address misaligned",
        5 => "Load access fault",
        6 => "Store/AMO address misaligned",
        7 => "Store/AMO access fault",
        8 => "Environment call from U-mode",
        9 => "Environment call from S-mode",
        10 => "Reserved",
        11 => "Environment call from M-mode",
        12 => "Instruction page fault",
        13 => "Load page fault",
        14 => "Reserved",
        15 => "Store/AMO page fault",
        _ => "UNKNOWN",
    };
    println!("exception '{}' mepc={:x}, mtval={:x}", code, mepc, mtval);
    println!("{:#x?}", _trap_frame);

    print_backtrace_addresses_internal(_trap_frame.s0 as u32, 0);
    loop {}
}

#[panic_handler]
fn panic_handler(info: &core::panic::PanicInfo) -> ! {
    unsafe {
        riscv::interrupt::disable();
    }
    writeln!(Uart, "{}", info).ok();
    print_backtrace_addresses();
    loop {}
}

fn print_backtrace_addresses() {
    let fp = unsafe {
        let mut _tmp: u32;
        asm!("mv {0}, x8", out(reg) _tmp);
        _tmp
    };

    print_backtrace_addresses_internal(fp, 2);
}

fn print_backtrace_addresses_internal(fp: u32, suppress: i32) {
    let mut fp = fp;
    let mut suppress = suppress;
    let mut old_address = 0;
    loop {
        unsafe {
            let address = (fp as *const u32).offset(-1).read(); // RA/PC
            fp = (fp as *const u32).offset(-2).read(); // next FP

            if old_address == address {
                break;
            }

            old_address = address;

            // currently this only supports code in flash
            if !(0x42000000..=0x42800000).contains(&address) {
                break;
            }

            if suppress == 0 {
                write!(Uart, "0x{:x} \r\n", address).ok();
            } else {
                suppress -= 1;
            }
        }
    }
}
