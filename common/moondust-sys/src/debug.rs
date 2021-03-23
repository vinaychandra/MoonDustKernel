use core::fmt::{self, Write};

use crate::syscall::Syscalls;

struct DebugPrinter {}

impl fmt::Write for DebugPrinter {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        let b = s.as_bytes();
        let scall = Syscalls::Debug {
            ptr: b.as_ptr() as u64,
            len: b.len() as u64,
        };
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
