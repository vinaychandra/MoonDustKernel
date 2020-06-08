#![no_std]
#![no_main]
#![feature(thread_local)]
#![feature(llvm_asm)]
#![feature(duration_constants)]

use alloc::boxed::Box;
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
    spawner.spawn(Task::new(example_task(41)));
    spawner.spawn(Task::new(crate::devices::timer::timer_task()));
    spawner.spawn(Task::new(timer_test()));
    spawner.spawn(Task::new(example_task(42)));
    executor.run();
}

/// This function is called on panic.
#[cfg(not(test))]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    kernel_error!("PANIC: {}", info);
    arch::hlt_loop()
}

async fn async_number(value: u8) -> u8 {
    value + 1
}

async fn example_task(value: u8) {
    let mut number = value;
    loop {
        number = async_number(number).await;
        kernel_info!("async number: {} {}", value, number);
        if number % 5 == 0 {
            break;
        }
    }
    kernel_info!("async done: {}", value);
}

async fn timer_test() {
    kernel_info!("Before timer");
    loop {
        let future = TimerFuture::new(core::time::Duration::new(5, 0));
        future.await;
        kernel_info!("Uptime: {:?}", crate::devices::timer::up_time());
    }
}
