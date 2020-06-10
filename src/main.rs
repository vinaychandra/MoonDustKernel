#![no_std]
#![no_main]
#![feature(thread_local)]
#![feature(llvm_asm)]
#![feature(async_closure)]
#![feature(duration_constants)]

use crate::sync::Mutex;
use alloc::{boxed::Box, sync::Arc};
use bootloader::{entry_point, BootInfo};
#[cfg(not(test))]
use core::panic::PanicInfo;
use devices::timer::TimerFuture;
use moondust_kernel::*;
use tasks::{executor::Executor, Task};

extern crate alloc;

#[thread_local]
pub static mut TEST: u8 = 9;

entry_point!(kernel_main);

/// Entry point for the Operating System.
#[no_mangle] // don't mangle the name of this function
fn kernel_main(boot_info: &'static BootInfo) -> ! {
    // Initialize logging so that data can be seen on screen
    moondust_kernel::initialize_logging();

    // Initialize architecture
    arch::init(boot_info);

    // test box
    let _test = Box::new(10u64);

    x86_64::instructions::interrupts::enable();

    let tls_val = unsafe { TEST };
    kernel_info!("TLS value is {}", tls_val);

    let mut executor = Executor::new();
    let spawner = executor.get_spawner();

    let m = Arc::new(Mutex::new(0));
    let m2 = m.clone();
    spawner.spawn(Task::new(crate::devices::timer::timer_task()));
    spawner.spawn(Task::new(mutex1(m)));
    spawner.spawn(Task::new(mutex2(m2)));

    executor.run();
}

/// This function is called on panic.
#[cfg(not(test))]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    kernel_error!("PANIC: {}", info);
    arch::hlt_loop()
}

async fn mutex1(m: Arc<Mutex<i32>>) {
    kernel_info!("1");
    let _lock = m.lock().await;
    kernel_info!("2");
    TimerFuture::new(core::time::Duration::new(1, 0)).await;
    kernel_info!("4");
}

async fn mutex2(m: Arc<Mutex<i32>>) {
    kernel_info!("3");
    let _lock = m.lock().await;
    kernel_info!("5");
}
