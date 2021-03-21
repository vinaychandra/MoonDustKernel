use log::Log;

pub struct UnifiedLogger {}

impl UnifiedLogger {
    pub const fn new() -> UnifiedLogger {
        UnifiedLogger {}
    }
}

impl Log for UnifiedLogger {
    fn enabled(&self, _metadata: &log::Metadata) -> bool {
        true
    }

    fn log(&self, record: &log::Record) {
        crate::arch::LOGGER.log(record);
    }

    fn flush(&self) {
        crate::arch::LOGGER.flush();
    }
}
