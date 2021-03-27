use log::Log;

use crate::arch::globals;

/// Common logging implementation for the kernel.
pub struct UnifiedLogger {}

impl UnifiedLogger {
    pub const fn new() -> UnifiedLogger {
        UnifiedLogger {}
    }
}

impl Log for UnifiedLogger {
    fn enabled(&self, metadata: &log::Metadata) -> bool {
        if globals::DEFAULT_LOG_LEVEL >= metadata.level() {
            return true;
        }

        if globals::EXTRA_LOGS.contains(&metadata.target()) {
            return true;
        }

        false
    }

    fn log(&self, record: &log::Record) {
        if self.enabled(record.metadata()) {
            // We default to the architecture defined logger.
            crate::arch::LOGGER.log(record);
        }
    }

    fn flush(&self) {
        crate::arch::LOGGER.flush();
    }
}
