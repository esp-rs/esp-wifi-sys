#![no_std]

#[cfg(feature = "esp32")]
pub use esp32_hal as hal;
#[cfg(feature = "esp32c2")]
pub use esp32c2_hal as hal;
#[cfg(feature = "esp32c3")]
pub use esp32c3_hal as hal;
#[cfg(feature = "esp32c6")]
pub use esp32c6_hal as hal;
#[cfg(feature = "esp32s2")]
pub use esp32s2_hal as hal;
#[cfg(feature = "esp32s3")]
pub use esp32s3_hal as hal;

#[macro_export]
macro_rules! rtc {
    ($peripherals: ident) => {{
        #[cfg(not(any(feature = "esp32c6")))]
        let mut rtc = Rtc::new($peripherals.RTC_CNTL);

        #[cfg(any(feature = "esp32c6"))]
        let mut rtc = Rtc::new($peripherals.LP_CLKRST);

        // Disable watchdog timers
        #[cfg(not(any(feature = "esp32", feature = "esp32s2")))]
        rtc.swd.disable();

        rtc.rwdt.disable();

        rtc
    }};
}

#[macro_export]
macro_rules! system {
    ($peripherals: ident) => {{
        #[cfg(not(any(feature = "esp32", feature = "esp32c6")))]
        let system = $peripherals.SYSTEM.split();
        #[cfg(feature = "esp32")]
        let system = $peripherals.DPORT.split();
        #[cfg(any(feature = "esp32c6"))]
        let system = $peripherals.PCR.split();

        system
    }};
}

#[macro_export]
macro_rules! clocks {
    ($system: ident) => {{
        #[cfg(feature = "esp32c3")]
        let clocks = ClockControl::configure($system.clock_control, CpuClock::Clock160MHz).freeze();
        #[cfg(feature = "esp32c2")]
        let clocks = ClockControl::configure($system.clock_control, CpuClock::Clock120MHz).freeze();
        #[cfg(feature = "esp32c6")]
        let clocks = ClockControl::configure($system.clock_control, CpuClock::Clock160MHz).freeze();
        #[cfg(any(feature = "esp32", feature = "esp32s3", feature = "esp32s2"))]
        let clocks = ClockControl::configure($system.clock_control, CpuClock::Clock240MHz).freeze();

        clocks
    }};
}

#[macro_export]
macro_rules! timer {
    ($peripherals: ident, $clocks: ident, $peripheral_clock_control: ident) => {{
        #[cfg(any(feature = "esp32c3", feature = "esp32c2", feature = "esp32c6"))]
        {
            let _unused = &mut $peripheral_clock_control;
            use hal::systimer::SystemTimer;
            SystemTimer::new($peripherals.SYSTIMER).alarm0
        }
        #[cfg(any(feature = "esp32", feature = "esp32s3", feature = "esp32s2"))]
        {
            use hal::timer::TimerGroup;
            TimerGroup::new($peripherals.TIMG1, &$clocks, &mut $peripheral_clock_control).timer0
        }
    }};
}

#[macro_export]
macro_rules! boot_button {
    ($peripherals: ident) => {{
        let io = IO::new($peripherals.GPIO, $peripherals.IO_MUX);
        #[cfg(any(feature = "esp32", feature = "esp32s2", feature = "esp32s3"))]
        let button = io.pins.gpio0.into_pull_down_input();
        #[cfg(any(feature = "esp32c2", feature = "esp32c3"))]
        let button = io.pins.gpio9.into_pull_down_input();
        button
    }};
}

#[macro_export]
macro_rules! get_bluetooth {
    ($peripherals: ident) => {{
        #[cfg(any(
            feature = "esp32",
            feature = "esp32s3",
            feature = "esp32c2",
            feature = "esp32c3"
        ))]
        let (_, bluetooth) = $peripherals.RADIO.split();
        #[cfg(any(feature = "esp32c6"))]
        let (_, bluetooth, _) = $peripherals.RADIO.split();
        #[cfg(any(feature = "esp32s2"))]
        compile_error!("Bluetooth is not supported");
        bluetooth
    }};
}

#[macro_export]
macro_rules! get_wifi {
    ($peripherals: ident) => {{
        #[cfg(any(
            feature = "esp32",
            feature = "esp32s3",
            feature = "esp32c2",
            feature = "esp32c3"
        ))]
        let (wifi, _) = $peripherals.RADIO.split();
        #[cfg(any(feature = "esp32s2"))]
        let wifi = $peripherals.RADIO.split();
        #[cfg(any(feature = "esp32c6"))]
        let (wifi, _, _) = $peripherals.RADIO.split();
        wifi
    }};
}

#[macro_export]
macro_rules! get_wifi_bluetooth {
    ($peripherals: ident) => {{
        #[cfg(any(
            feature = "esp32",
            feature = "esp32s3",
            feature = "esp32c2",
            feature = "esp32c3"
        ))]
        let (wifi, bluetooth) = $peripherals.RADIO.split();
        #[cfg(any(feature = "esp32c6"))]
        let (wifi, bluetooth, _) = $peripherals.RADIO.split();
        (wifi, bluetooth)
    }};
}

#[cfg(any(feature = "esp32c2", feature = "esp32c3"))]
pub type BootButton = crate::hal::gpio::Gpio9<crate::hal::gpio::Input<crate::hal::gpio::PullDown>>;
#[cfg(any(feature = "esp32", feature = "esp32s2", feature = "esp32s3"))]
pub type BootButton = crate::hal::gpio::Gpio0<crate::hal::gpio::Input<crate::hal::gpio::PullDown>>;

#[cfg(feature = "esp32c3")]
pub const SOC_NAME: &str = "ESP32-C3";
#[cfg(feature = "esp32c2")]
pub const SOC_NAME: &str = "ESP32-C2";
#[cfg(feature = "esp32")]
pub const SOC_NAME: &str = "ESP32";
#[cfg(feature = "esp32s3")]
pub const SOC_NAME: &str = "ESP32-S3";
