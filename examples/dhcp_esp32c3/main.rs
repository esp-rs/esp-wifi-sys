#![no_std]
#![no_main]
#![feature(c_variadic)]
#![feature(const_mut_refs)]

use core::{arch::asm, fmt::Write};

use esp32c3_hal::{interrupt::TrapFrame, pac::Peripherals, RtcCntl};
use esp_wifi::wifi::initialize;
use esp_wifi::wifi::utils::create_network_stack;
use esp_wifi::{
    binary, compat, println,
    wifi::{self, wifi_connect},
};
use esp_wifi::{create_network_stack_storage, network_stack_storage, Uart};
use riscv_rt::entry;

use embedded_nal::SocketAddrV4;
use embedded_nal::TcpClientStack;

const SSID: &str = env!("SSID");
const PASSWORD: &str = env!("PASSWORD");

#[entry]
fn main() -> ! {
    let mut peripherals = Peripherals::take().unwrap();

    let mut rtc_cntl = RtcCntl::new(peripherals.RTC_CNTL);

    // Disable watchdog timers
    rtc_cntl.set_super_wdt_enable(false);
    rtc_cntl.set_wdt_enable(false);

    init_logger();

    initialize(
        &mut peripherals.SYSTIMER,
        &mut peripherals.INTERRUPT_CORE0,
        peripherals.RNG,
    )
    .unwrap();

    println!("Call wifi_start_scan");
    let res = wifi::wifi_start_scan();
    println!("wifi_start_scan returned {}", res);
    print_scan_result();
    println!("\n\n\n\n");

    let mut storage = create_network_stack_storage!(3, 8, 1);
    let mut network_stack = create_network_stack(network_stack_storage!(storage));

    println!("Call wifi_connect");
    let res = wifi_connect(SSID, PASSWORD);
    println!("wifi_connect returned {}", res);

    loop {
        if wifi::is_connected() {
            break;
        }
    }

    println!("Start busy loop on main");
    let mut stage = 0;
    let mut socket = None;
    let mut idx = 0;
    let mut buffer = [0u8; 8000];
    let mut waiter = 50000;
    loop {
        network_stack.poll().ok();

        if let Some(ipv4_addr) = network_stack.interface().ipv4_addr() {
            if !ipv4_addr.is_unspecified() {
                match stage {
                    0 => {
                        println!("My IP is {}", ipv4_addr);
                        println!("Lets connect");
                        let mut sock = network_stack.socket().unwrap();
                        let addr =
                            SocketAddrV4::new(embedded_nal::Ipv4Addr::new(142, 250, 185, 115), 80);
                        network_stack.connect(&mut sock, addr.into()).unwrap();

                        socket = Some(sock);
                        stage = 1;
                        println!("Lets send");
                    }
                    1 => {
                        if network_stack
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
                        if let Ok(s) =
                            network_stack.receive(&mut socket.unwrap(), &mut buffer[idx..])
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
                        network_stack.close(socket.unwrap()).ok();
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
