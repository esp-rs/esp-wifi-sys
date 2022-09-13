pub mod c_types;

#[allow(improper_ctypes)]
#[cfg_attr(feature = "esp32c3", path = "include_esp32c3.rs")]
#[cfg_attr(feature = "esp32", path = "include_esp32.rs")]
pub mod include;
