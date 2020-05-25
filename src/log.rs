use crate::arch::log::BOOTSTRAP_LOGGER;
use log::LevelFilter;

/// Initialize the bootstrap logger. This is dependant on the architecture
/// based static `BOOTSTRAP_LOGGER` that implements `Log`.
pub fn init_bootstrap_log() {
    let bootstrap_logger: &dyn log::Log = &*BOOTSTRAP_LOGGER;
    log::set_logger(bootstrap_logger)
        .map(|()| log::set_max_level(LevelFilter::Trace))
        .unwrap();
}

/// Kernel log support.
/// This takes the module as the first parameter. This module can be used
/// for conditional compilation.
/// This also supports logs without a module. Those logs are not filtered.
/// Those type of logs are stored using the target "default".
#[macro_export]
macro_rules! kernel_log {
    (panic, $lvl:expr, $($arg:tt)+) => {
        #[cfg(feature = "log-panic")]
        ::log::log!(target: "panic", $lvl, "{}\n", format_args!($($arg)+))
    };

    (memory, $lvl:expr, $($arg:tt)+) => {
        #[cfg(feature = "log-memory")]
        ::log::log!(target: "memory", $lvl, "{}\n", format_args!($($arg)+))
    };

    (scheduler, $lvl:expr, $($arg:tt)+) => {
        #[cfg(feature = "log-scheduler")]
        ::log::log!(target: "scheduler", $lvl, "{}\n", format_args!($($arg)+))
    };

    // Default logs.
    ($lvl:expr, $($arg:tt)+) => (::log::log!(target: "default", $lvl, "{}\n", format_args!($($arg)+)));
}

/// Trace logs for kernel.
#[macro_export]
macro_rules! kernel_trace {
    ($module:ident, $($arg:tt)+) => (kernel_log!($module, ::log::Level::Trace, $($arg)+));
    ($($arg:tt)+) => (kernel_log!(::log::Level::Trace, $($arg)+));
}

/// Debug logs for kernel.
#[macro_export]
macro_rules! kernel_debug {
    ($module:ident, $($arg:tt)+) => (kernel_log!($module, ::log::Level::Debug, $($arg)+));
    ($($arg:tt)+) => (kernel_log!(::log::Level::Debug, $($arg)+));
}

/// Info logs for kernel.
#[macro_export]
macro_rules! kernel_info {
    ($module:ident, $($arg:tt)+) => (kernel_log!($module, ::log::Level::Info, $($arg)+));
    ($($arg:tt)+) => (kernel_log!(::log::Level::Info, $($arg)+));
}

/// Warn logs for kernel.
#[macro_export]
macro_rules! kernel_warn {
    ($module:ident, $($arg:tt)+) => (kernel_log!($module, ::log::Level::Warn, $($arg)+));
    ($($arg:tt)+) => (kernel_log!(::log::Level::Warn, $($arg)+));
}

/// Error logs for kernel.
#[macro_export]
macro_rules! kernel_error {
    ($module:ident, $($arg:tt)+) => (kernel_log!($module, ::log::Level::Error, $($arg)+));
    ($($arg:tt)+) => (kernel_log!(::log::Level::Error, $($arg)+));
}
