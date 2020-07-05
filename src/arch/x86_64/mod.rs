pub mod gdt;
pub mod globals;
pub mod interrupts;
pub mod memory;
pub mod serial;

use crate::{
    bootboot,
    common::memory::{
        allocator::{boot_frame_allocator::BootFrameAllocator, physical_memory_allocator},
        heap,
        stack::Stack,
    },
};
use core::cell::UnsafeCell;
use log::LevelFilter;
use serial::SerialLogger;
use spin::Mutex;
use x86_64::{registers::control::EferFlags, structures::paging::OffsetPageTable, VirtAddr};

static LOGGER: SerialLogger = SerialLogger;

pub fn initialize_architecture_bsp() -> ! {
    // Initialize logging.
    log::set_logger(&LOGGER)
        .map(|()| log::set_max_level(LevelFilter::Info))
        .expect("Setting logger failed");

    info!(target: "initialize_architecture", "Initializing x86_64 architecture.");

    {
        let mut efer = x86_64::registers::model_specific::Efer::read();
        efer |= EferFlags::NO_EXECUTE_ENABLE;
        efer |= EferFlags::SYSTEM_CALL_EXTENSIONS;
        unsafe {
            x86_64::registers::model_specific::Efer::write(efer);
        }
    }

    info!(target: "initialize_architecture", "Setting up memory.");
    let mut frame_allocator = {
        let entries = unsafe { bootboot::bootboot.get_mmap_entries() };
        UnsafeCell::new(BootFrameAllocator::new(entries))
    };

    let mut mapper = memory::init_bsp();
    let new_stack = Stack::bsp_kernel_stack(&mut mapper, &mut frame_allocator).unwrap();
    {
        let mut a = MEM.lock();
        *a = Some((mapper, frame_allocator));
    }

    // Switch to a new stack
    info!(target: "initialize_architecture", "Switching to bsp stack");
    new_stack.switch_to();
    unsafe { asm!("jmp {}", sym initialize_architecture_bsp_stack) };

    error!("Unexpected continuation of stack switching.");
    loop {}
}

static MEM: Mutex<Option<(OffsetPageTable, UnsafeCell<BootFrameAllocator>)>> = Mutex::new(None);

pub fn initialize_architecture_bsp_stack() -> ! {
    let (mut mapper, mut frame_allocator) = MEM.lock().take().unwrap();
    {
        info!(target: "initialize_architecture_bsp", "Initializing heap");
        heap::initialize_heap(&mut mapper, &mut frame_allocator)
            .expect("Heap initialization failed.");
        info!(target: "initialize_architecture_bsp", "heap initialization complete");
    }

    {
        info!(target: "initialize_architecture_bsp", "Initializing physical memory provider");
        physical_memory_allocator::initialize_physical_memory_allocator(
            &mut frame_allocator.into_inner(),
            4096,
        );
        info!(target: "initialize_architecture_bsp", "Initialized physical memory provider");
    }

    {
        info!(target: "initialize_architecture_bsp", "Initializing TLS");
        memory::initialize_tls();
        info!(target: "initialize_architecture_bsp", "TLS Initialized");
    }

    {
        info!(target: "initialize_architecture_bsp", "Initialize GDT");
        gdt::init();
        info!(target: "initialize_architecture_bsp", "GDT ready");
    }

    {
        info!(target: "initialize_architecture_bsp", "Initialize interrupts");
        interrupts::initialize(VirtAddr::new(globals::MEM_MAP_LOCATION));
        info!(target: "initialize_architecture_bsp", "interrupts ready");
    }

    crate::main_bsp();
}

/// Halt the CPU until next interrupt.
pub fn hlt_loop() -> ! {
    loop {
        x86_64::instructions::hlt();
    }
}
