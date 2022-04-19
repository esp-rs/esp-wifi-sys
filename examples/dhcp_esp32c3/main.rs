#![no_std]
#![no_main]
#![feature(c_variadic)]
#![feature(const_mut_refs)]

use core::{arch::asm, fmt::Write};

use embedded_svc::wifi::{
    ClientConfiguration, ClientConnectionStatus, ClientIpStatus, ClientStatus, Configuration,
    Status, Wifi,
};
use esp32c3_hal::{interrupt::TrapFrame, pac::Peripherals, RtcCntl};
use esp_wifi::println;
use esp_wifi::wifi::initialize;
use esp_wifi::wifi::utils::create_network_stack;
use esp_wifi::{create_network_stack_storage, network_stack_storage, Uart};
use riscv_rt::entry;

use embedded_nal::SocketAddrV4;
use embedded_nal::TcpClientStack;

extern crate alloc;

const SSID: &str = env!("SSID");
const PASSWORD: &str = env!("PASSWORD");

#[entry]
fn main() -> ! {
    let mut peripherals = Peripherals::take().unwrap();

    let mut rtc_cntl = RtcCntl::new(peripherals.RTC_CNTL);

    // Disable watchdog timers
    rtc_cntl.set_super_wdt_enable(false);
    rtc_cntl.set_wdt_enable(false);

    let mut storage = create_network_stack_storage!(3, 8, 1);
    let network_stack = create_network_stack(network_stack_storage!(storage));

    let mut wifi_interface = esp_wifi::wifi_interface::Wifi::new(network_stack);

    init_logger();

    initialize(
        &mut peripherals.SYSTIMER,
        &mut peripherals.INTERRUPT_CORE0,
        peripherals.RNG,
    )
    .unwrap();

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
