use bootloader::BootInfo;
use x86_64::VirtAddr;

mod _console_vga;
mod gdt;
pub mod interrupts;
pub mod log;
mod memory;

pub fn hlt_loop() -> ! {
    loop {
        x86_64::instructions::hlt();
    }
}

/// Architecture level initialization.
pub fn init(boot_info: &'static BootInfo) {
    gdt::init();

    interrupts::init_idt();

    // Setup memory and heap
    let phys_mem_offset = VirtAddr::new(boot_info.physical_memory_offset);
    let mut mapper = unsafe { memory::init(phys_mem_offset) };
    let mut frame_allocator =
        memory::boot_frame_allocator::BootFrameAllocator::new(&boot_info.memory_map);

    memory::heap::init_heap(&mut mapper, &mut frame_allocator).expect("Heap allocation failed");
}
