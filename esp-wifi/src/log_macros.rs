#[cfg(feature = "log")]
pub use log;
#[cfg(not(feature = "log"))]
pub use noop as log;

#[cfg(feature = "log-defmt")]
pub use defmt;
#[cfg(not(feature = "log-defmt"))]
pub use noop as defmt;

#[cfg(any(not(feature = "log-defmt"), not(feature = "log")))]
pub mod noop {
    #[macro_export]
    macro_rules! noop {
        ($($args:expr),*) => {
            $(_=&$args;)*
        };
    }

    pub use noop as trace;
    pub use noop as debug;
    pub use noop as info;
    pub use noop as warn;
    pub use noop as error;
}

#[macro_export]
macro_rules! trace {
    ($($args:expr),* $(,)?) => {{
        $crate::log_macros::log::trace!($($args),*);
        $crate::log_macros::defmt::trace!($($args),*);
    }}
}

#[macro_export]
macro_rules! debug {
    ($($args:expr),* $(,)?) => {{
        $crate::log_macros::log::debug!($($args),*);
        $crate::log_macros::defmt::debug!($($args),*);
    }}
}

#[macro_export]
macro_rules! info {
    ($($args:expr),* $(,)?) => {{
        $crate::log_macros::log::info!($($args),*);
        $crate::log_macros::defmt::info!($($args),*);
    }}
}

#[macro_export]
macro_rules! warn {
    ($($args:expr),* $(,)?) => {{
        $crate::log_macros::log::warn!($($args),*);
        $crate::log_macros::defmt::warn!($($args),*);
    }}
}

#[macro_export]
macro_rules! error {
    ($($args:expr),* $(,)?) => {{
        $crate::log_macros::log::error!($($args),*);
        $crate::log_macros::defmt::error!($($args),*);
    }}
}
