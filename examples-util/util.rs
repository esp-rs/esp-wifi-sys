#![allow(unused)]

pub use esp_hal as hal;

#[cfg(any(
    feature = "esp32c2",
    feature = "esp32c3",
    feature = "esp32c6",
    feature = "esp32h2"
))]
pub type BootButton = crate::hal::gpio::Gpio9<crate::hal::gpio::Input<crate::hal::gpio::PullDown>>;
#[cfg(any(feature = "esp32", feature = "esp32s2", feature = "esp32s3"))]
pub type BootButton = crate::hal::gpio::Gpio0<crate::hal::gpio::Input<crate::hal::gpio::PullDown>>;

#[cfg(feature = "esp32c3")]
pub const SOC_NAME: &str = "ESP32-C3";
#[cfg(feature = "esp32c2")]
pub const SOC_NAME: &str = "ESP32-C2";
#[cfg(feature = "esp32c6")]
pub const SOC_NAME: &str = "ESP32-C6";
#[cfg(feature = "esp32h2")]
pub const SOC_NAME: &str = "ESP32-H2";
#[cfg(feature = "esp32")]
pub const SOC_NAME: &str = "ESP32";
#[cfg(feature = "esp32s3")]
pub const SOC_NAME: &str = "ESP32-S3";
