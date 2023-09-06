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
