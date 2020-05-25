use super::_console_vga::{Color, ColorCode, VgaImmutableWriter, CONSOLE_DISPLAY_GLOBAL};
use log::Level;

pub static BOOTSTRAP_LOGGER_INSTANCE: BootstrapLogger = BootstrapLogger;
pub static BOOTSTRAP_LOGGER: &BootstrapLogger = &BOOTSTRAP_LOGGER_INSTANCE;

/// The logger that is used to write logs during Kernel bootstrapping.
/// This exists because the drivers for display are not loaded at the beginning
/// of the kernel.
pub struct BootstrapLogger;

impl log::Log for BootstrapLogger {
    fn enabled(&self, _metadata: &log::Metadata) -> bool {
        true
    }

    fn flush(&self) {}

    fn log(&self, record: &log::Record) {
        let instance: &'static VgaImmutableWriter = &*CONSOLE_DISPLAY_GLOBAL;

        match record.level() {
            Level::Error => instance.concurrent_write_color_fmt(
                ColorCode::new(Color::Red, Color::Black),
                format_args!(
                    "[{}:{}] {}",
                    record.file().unwrap(),
                    record.line().unwrap(),
                    *record.args()
                ),
            ),
            Level::Warn => instance.concurrent_write_color_fmt(
                ColorCode::new(Color::Yellow, Color::Black),
                format_args!(
                    "[{}:{}] {}",
                    record.file().unwrap(),
                    record.line().unwrap(),
                    *record.args()
                ),
            ),
            Level::Info => instance.concurrent_write_color_fmt(
                ColorCode::new(Color::White, Color::Black),
                format_args!("{}", *record.args()),
            ),
            Level::Debug => instance.concurrent_write_color_fmt(
                ColorCode::new(Color::DarkGray, Color::Black),
                format_args!("{}", *record.args()),
            ),
            Level::Trace => instance.concurrent_write_color_fmt(
                ColorCode::new(Color::LightGray, Color::Black),
                format_args!("{}", *record.args()),
            ),
        }
    }
}
