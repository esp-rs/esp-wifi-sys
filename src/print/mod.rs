#[macro_export]
macro_rules! println {
    ($($arg:tt)*) => {
        #[allow(unused_unsafe)]
        esp_wifi::critical_section::with(|_| {
        unsafe {
            use core::fmt::Write;
            writeln!(crate::Uart, $($arg)*).ok();
          }
        });
    };
}

#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => {
        #[allow(unused_unsafe)]
        esp_wifi::critical_section::with(|_| {
        unsafe {
            use core::fmt::Write;
            write!(crate::Uart, $($arg)*).ok();
          }
        });
    };
}
