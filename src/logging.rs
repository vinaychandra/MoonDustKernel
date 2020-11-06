use core::sync::atomic::{AtomicBool, Ordering};
use log::Log;

use crate::common::graphics::gui::GuiLogger;

pub struct UnifiedLogger {
    gui_logger: (AtomicBool, GuiLogger),
}

impl UnifiedLogger {
    pub const fn new() -> UnifiedLogger {
        UnifiedLogger {
            gui_logger: (AtomicBool::new(false), GuiLogger),
        }
    }

    pub fn enable_gui_logger(&self) {
        self.gui_logger.0.store(true, Ordering::Relaxed);
    }
}

impl Log for UnifiedLogger {
    fn enabled(&self, _metadata: &log::Metadata) -> bool {
        true
    }

    fn log(&self, record: &log::Record) {
        if self.gui_logger.0.load(Ordering::Relaxed) {
            self.gui_logger.1.log(record);
            self.gui_logger.1.flush();
        }

        crate::arch::LOGGER.log(record);
        crate::arch::LOGGER.flush();
        self.flush();
    }

    fn flush(&self) {}
}
