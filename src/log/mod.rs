#[derive(PartialEq, PartialOrd, Clone, Copy)]
pub enum LogLevel {
    None,
    Debug,
    Verbose,
    Trace,
}

pub const LOG_LEVEL: LogLevel = LogLevel::None;

#[macro_export]
macro_rules! trace {
    ($($arg:tt)*) => {
        #[allow(unused_unsafe)]
        if unsafe { $crate::log::LOG_LEVEL } >= $crate::log::LogLevel::Trace {
            critical_section::with(|_| {
                use core::fmt::Write;

                unsafe {
                    write!(crate::Uart, "{}: ", $crate::preempt::current_task()).ok();
                }
                write!(crate::Uart, $($arg)*).ok();
                write!(crate::Uart, "\r\n").ok();
            });
        }
    };
}

#[macro_export]
macro_rules! verbose {
    ($($arg:tt)*) => {
        #[allow(unused_unsafe)]
        if $crate::log::LOG_LEVEL >= $crate::log::LogLevel::Verbose {
            critical_section::with(|_| {
                use core::fmt::Write;

                unsafe {
                    write!(crate::Uart, "{}: ", $crate::preempt::current_task()).ok();
                }
                write!(crate::Uart, $($arg)*).ok();
                write!(crate::Uart, "\r\n").ok();
            });
        }
    };
}

#[macro_export]
macro_rules! debug {
    ($($arg:tt)*) => {
        #[allow(unused_unsafe)]
        if $crate::log::LOG_LEVEL >= $crate::log::LogLevel::Debug {
            critical_section::with(|_| {
                use core::fmt::Write;

                unsafe {
                    write!(crate::Uart, "{}: ", $crate::preempt::current_task()).ok();
                }
                write!(crate::Uart, $($arg)*).ok();
                write!(crate::Uart, "\r\n").ok();
            });
        }
    };
}

#[macro_export]
macro_rules! println {
    ($($arg:tt)*) => {
        #[allow(unused_unsafe)]
        unsafe {
            use core::fmt::Write;

            write!(crate::Uart, $($arg)*).ok();
            write!(crate::Uart, "\r\n").ok();
        }
    };
}

#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => {
        use core::fmt::Write;

        #[allow(unused_unsafe)]
        write!(crate::Uart, $($arg)*).ok();
    };
}
