//! x86_64 specific startup logic.

pub mod devices;
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
        stack::{self, Stack},
    },
};
use core::cell::UnsafeCell;
use log::LevelFilter;
use serial::SerialLogger;
use spin::Mutex;
use x86_64::{registers::control::EferFlags, structures::paging::OffsetPageTable, VirtAddr};

/// Logger that uses serial to output logs.
/// Architecture level logs for x86_64.
pub static LOGGER: SerialLogger = SerialLogger;

static MEM: Mutex<Option<(Option<OffsetPageTable>, UnsafeCell<BootFrameAllocator>)>> =
    Mutex::new(None);

/// Initialization on bootstrap processor.
pub fn initialize_architecture_bsp() -> ! {
    info!(target: "initialize_architecture", "Initializing x86_64 architecture.");

    // This enables syscall extensions on x86_64
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
        BootFrameAllocator::new(entries)
    };

    // 2 pages for initial bootstrapping. This acts as an intermediate step.
    // We need this for setting up for the main stacks but the bootloader only provdes 1K in memory.
    let level_2_addr = frame_allocator
        .alloc(8096, 4096)
        .expect("Cannot allocate bootstrap stack")
        + 8096
        - globals::STACK_ALIGN;

    let frame_allocator = UnsafeCell::new(frame_allocator);
    {
        let mut a = MEM.lock();
        *a = Some((None, frame_allocator));
    }

    // Switch to level 2.
    unsafe {
        asm!("
        mov rsp, {0}
        mov rbp, {0}
        jmp {1}
        ", in(reg) level_2_addr, sym initialize_architecture_bsp2);
    }

    error!("Unexpected continuation of stack switching.");
    loop {}
}

/// Level 2 initializing.
/// This creates a memory map in higher half and then jumps to it.
/// There is a need for this because it requires more memory than the
/// bootloader provides.
pub fn initialize_architecture_bsp2() {
    // Initialize logging.
    log::set_logger(&crate::KERNEL_LOGGER)
        .map(|()| log::set_max_level(LevelFilter::Info))
        .expect("Setting logger failed");

    let (_, mut frame_allocator) = MEM.lock().take().unwrap();
    let mut mapper = memory::init_bsp(&mut frame_allocator);
    let new_stack = Stack::bsp_kernel_stack(&mut mapper, &mut frame_allocator).unwrap();

    {
        let mut a = MEM.lock();
        *a = Some((Some(mapper), frame_allocator));
    }

    // Switch to a new stack
    info!(target: "initialize_architecture", "Switching to bsp stack");
    new_stack.switch_to();
    unsafe { asm!("jmp {}", sym initialize_architecture_bsp_stack) };
}

/// Initialize on the main stack. This uses the final stack used by the kernel.
pub fn initialize_architecture_bsp_stack() -> ! {
    let (mapper, mut frame_allocator) = MEM.lock().take().unwrap();
    let mut mapper = mapper.unwrap();
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

    {
        info!(target: "initialize_architecture_bsp", "Initialize stack provider");
        let frame_allocator = physical_memory_allocator::get_physical_memory_allocator();
        stack::initialize_stack_provider_bsp(&mut mapper, frame_allocator);
        info!(target: "initialize_architecture_bsp", "Stack provider initialized");
    }

    crate::main_bsp();
}

/// Halt the CPU until next interrupt.
pub fn hlt_loop() -> ! {
    loop {
        x86_64::instructions::hlt();
    }
}

pub fn enable_interrupts_and_halt() {
    x86_64::instructions::interrupts::enable_interrupts_and_hlt();
}

pub fn disable_interrupts() {
    x86_64::instructions::interrupts::disable();
}

pub fn enable_interrupts() {
    x86_64::instructions::interrupts::enable();
}
