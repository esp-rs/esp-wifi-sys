#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]

use core::cell::RefCell;

use embassy_executor::_export::StaticCell;
use embassy_sync::blocking_mutex::NoopMutex;
use examples_util::hal;

use embassy_executor::Executor;
use embassy_time::{Duration, Ticker};
use esp_backtrace as _;

use esp_println::println;
use esp_wifi::esp_now::{EspNowManager, EspNowReceiver, EspNowSender, PeerInfo, BROADCAST_ADDRESS};
use esp_wifi::{initialize, EspWifiInitFor};
use hal::clock::{ClockControl, CpuClock};
use hal::Rng;
use hal::{embassy, peripherals::Peripherals, prelude::*, timer::TimerGroup, Rtc};

#[cfg(any(feature = "esp32c3", feature = "esp32c2", feature = "esp32c6"))]
use hal::system::SystemExt;

#[embassy_executor::task]
async fn broadcaster(sender: &'static NoopMutex<RefCell<EspNowSender<'static>>>) {
    let mut ticker = Ticker::every(Duration::from_secs(1));
    loop {
        ticker.next().await;

        println!("Send Broadcast...");
        let status = sender.lock(|sender| {
            sender
                .borrow_mut()
                .send(&BROADCAST_ADDRESS, b"Hello.")
                .unwrap()
                .wait()
        });
        println!("Send broadcast status: {:?}", status);
    }
}

#[embassy_executor::task]
async fn sayhello(
    manager: &'static EspNowManager<'static>,
    sender: &'static NoopMutex<RefCell<EspNowSender<'static>>>,
) {
    let mut ticker = Ticker::every(Duration::from_millis(500));
    loop {
        ticker.next().await;
        let peer = match manager.fetch_peer(false) {
            Ok(peer) => peer,
            Err(_) => {
                if let Ok(peer) = manager.fetch_peer(true) {
                    peer
                } else {
                    continue;
                }
            }
        };

        println!("Send hello to peer {:?}", peer.peer_address);
        let status = sender.lock(|sender| {
            sender
                .borrow_mut()
                .send(&peer.peer_address, b"Hello Peer.")
                .unwrap()
                .wait()
        });
        println!("Send hello status: {:?}", status);
    }
}

#[embassy_executor::task]
async fn listener(manager: &'static EspNowManager<'static>, mut receiver: EspNowReceiver<'static>) {
    loop {
        let r = receiver.receive_async().await;
        println!("Received {:?}", r.get_data());
        if r.info.dst_address == BROADCAST_ADDRESS {
            if !manager.peer_exists(&r.info.src_address) {
                manager
                    .add_peer(PeerInfo {
                        peer_address: r.info.src_address,
                        lmk: None,
                        channel: None,
                        encrypt: false,
                    })
                    .unwrap();
                println!("Added peer {:?}", r.info.src_address);
            }
        }
    }
}

static EXECUTOR: StaticCell<Executor> = StaticCell::new();
static ESP_NOW_MANAGER: StaticCell<EspNowManager<'static>> = StaticCell::new();
static ESP_NOW_SENDER: StaticCell<NoopMutex<RefCell<EspNowSender<'static>>>> = StaticCell::new();

#[entry]
fn main() -> ! {
    #[cfg(feature = "log")]
    esp_println::logger::init_logger(log::LevelFilter::Info);

    let peripherals = Peripherals::take();

    let system = examples_util::system!(peripherals);
    let mut peripheral_clock_control = system.peripheral_clock_control;
    let clocks = examples_util::clocks!(system);
    examples_util::rtc!(peripherals);

    let timer = examples_util::timer!(peripherals, clocks, peripheral_clock_control);
    let init = initialize(
        EspWifiInitFor::Wifi,
        timer,
        Rng::new(peripherals.RNG),
        system.radio_clock_control,
        &clocks,
    )
    .unwrap();

    let wifi = examples_util::get_wifi!(peripherals);
    let esp_now = esp_wifi::esp_now::EspNow::new(&init, wifi).unwrap();
    println!("esp-now version {}", esp_now.get_version().unwrap());

    let timer_group0 = TimerGroup::new(peripherals.TIMG0, &clocks, &mut peripheral_clock_control);
    embassy::init(&clocks, timer_group0.timer0);
    let executor = EXECUTOR.init(Executor::new());

    let (manager, sender, receiver) = esp_now.split();
    let manager = ESP_NOW_MANAGER.init(manager);
    let sender: &'static _ = ESP_NOW_SENDER.init(NoopMutex::new(RefCell::new(sender)));

    executor.run(|spawner| {
        spawner.spawn(listener(manager, receiver)).ok();
        spawner.spawn(broadcaster(sender)).ok();
        spawner.spawn(sayhello(manager, sender)).ok();
    })
}
