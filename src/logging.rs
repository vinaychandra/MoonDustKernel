use core::sync::atomic::{AtomicBool, Ordering};
use log::Log;

use crate::{common::graphics::gui::GuiLogger, sync::signal::Signal};

pub struct UnifiedLogger {
    gui_logger: (AtomicBool, GuiLogger),
    signal: Signal,
}

impl UnifiedLogger {
    pub const fn new() -> UnifiedLogger {
        UnifiedLogger {
            gui_logger: (AtomicBool::new(false), GuiLogger),
            signal: Signal::new(),
        }
    }

    pub fn enable_gui_logger(&self) {
        self.gui_logger.0.store(true, Ordering::Relaxed);
    }

    pub async fn process_gui_logs(&self) -> u8 {
        info!("Starting GUI Log flushing");
        loop {
            self.flush();
            self.signal.wait_async().await;
        }
    }
}

impl Log for UnifiedLogger {
    fn enabled(&self, _metadata: &log::Metadata) -> bool {
        true
    }

    fn log(&self, record: &log::Record) {
        crate::arch::LOGGER.log(record);
        crate::arch::LOGGER.flush();

        if self.gui_logger.0.load(Ordering::Relaxed) {
            self.gui_logger.1.log(record);
            self.signal.signal();
        }
    }

    fn flush(&self) {
        if self.gui_logger.0.load(Ordering::Relaxed) {
            self.gui_logger.1.flush();
        }
    }
}
