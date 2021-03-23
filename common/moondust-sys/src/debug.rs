use core::fmt::{self, Write};

use crate::syscall::Syscalls;

struct DebugPrinter {}

impl fmt::Write for DebugPrinter {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        let scall = Syscalls::Debug { data: s };
        scall.invoke();
        Ok(())
    }
}

#[doc(hidden)]
pub fn _debug(args: ::core::fmt::Arguments) {
    let mut a = DebugPrinter {};
    a.write_fmt(args).unwrap();
}

/// Prints to the host through the debug interface.
#[macro_export]
macro_rules! debug_print {
    ($($arg:tt)*) => {
        $crate::debug::_debug(format_args!($($arg)*));
    };
}
