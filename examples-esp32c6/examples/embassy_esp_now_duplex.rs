#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]

use embassy_executor::_export::StaticCell;
use embassy_sync::blocking_mutex::raw::NoopRawMutex;
use embassy_sync::mutex::Mutex;
#[path = "../../examples-util/util.rs"]
mod examples_util;
use examples_util::hal;

use embassy_executor::Executor;
use embassy_time::{Duration, Ticker};
use esp_backtrace as _;

use esp_println::println;
use esp_wifi::esp_now::{EspNowManager, EspNowReceiver, EspNowSender, PeerInfo, BROADCAST_ADDRESS};
use esp_wifi::{initialize, EspWifiInitFor};
use hal::clock::ClockControl;
use hal::Rng;
use hal::{
    embassy, peripherals::Peripherals, prelude::*, systimer::SystemTimer, timer::TimerGroup,
};

#[embassy_executor::task]
async fn broadcaster(sender: &'static Mutex<NoopRawMutex, EspNowSender<'static>>) {
    let mut ticker = Ticker::every(Duration::from_secs(1));
    loop {
        ticker.next().await;

        println!("Send Broadcast...");
        let mut sender = sender.lock().await;
        let status = sender.send_async(&BROADCAST_ADDRESS, b"Hello.").await;
        println!("Send broadcast status: {:?}", status);
    }
}

#[embassy_executor::task]
async fn sayhello(
    manager: &'static EspNowManager<'static>,
    sender: &'static Mutex<NoopRawMutex, EspNowSender<'static>>,
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
        let mut sender = sender.lock().await;
        let status = sender.send_async(&peer.peer_address, b"Hello Peer.").await;
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
static ESP_NOW_SENDER: StaticCell<Mutex<NoopRawMutex, EspNowSender<'static>>> = StaticCell::new();

#[entry]
fn main() -> ! {
    #[cfg(feature = "log")]
    esp_println::logger::init_logger(log::LevelFilter::Info);

    let peripherals = Peripherals::take();

    let mut system = peripherals.PCR.split();
    let clocks = ClockControl::max(system.clock_control).freeze();

    let timer = SystemTimer::new(peripherals.SYSTIMER).alarm0;
    let init = initialize(
        EspWifiInitFor::Wifi,
        timer,
        Rng::new(peripherals.RNG),
        system.radio_clock_control,
        &clocks,
    )
    .unwrap();

    let (wifi, ..) = peripherals.RADIO.split();
    let esp_now = esp_wifi::esp_now::EspNow::new(&init, wifi).unwrap();
    println!("esp-now version {}", esp_now.get_version().unwrap());

    let timer_group0 = TimerGroup::new(
        peripherals.TIMG0,
        &clocks,
        &mut system.peripheral_clock_control,
    );
    embassy::init(&clocks, timer_group0.timer0);
    let executor = EXECUTOR.init(Executor::new());

    let (manager, sender, receiver) = esp_now.split();
    let manager = ESP_NOW_MANAGER.init(manager);
    let sender: &'static _ = ESP_NOW_SENDER.init(Mutex::new(sender));

    executor.run(|spawner| {
        spawner.spawn(listener(manager, receiver)).ok();
        spawner.spawn(broadcaster(sender)).ok();
        spawner.spawn(sayhello(manager, sender)).ok();
    })
}
