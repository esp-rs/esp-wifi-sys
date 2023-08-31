#[cfg(feature = "log")]
pub use log;
#[cfg(not(feature = "log"))]
pub use noop as log;

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
    }}
}

#[macro_export]
macro_rules! debug {
    ($($args:expr),* $(,)?) => {{
        $crate::log_macros::log::debug!($($args),*);
    }}
}

#[macro_export]
macro_rules! info {
    ($($args:expr),* $(,)?) => {{
        $crate::log_macros::log::info!($($args),*);
    }}
}

#[macro_export]
macro_rules! warn {
    ($($args:expr),* $(,)?) => {{
        $crate::log_macros::log::warn!($($args),*);
    }}
}

#[macro_export]
macro_rules! error {
    ($($args:expr),* $(,)?) => {{
        $crate::log_macros::log::error!($($args),*);
    }}
}
