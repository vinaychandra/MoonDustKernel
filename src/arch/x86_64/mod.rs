use crate::*;
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

    let phys_mem_offset = VirtAddr::new(boot_info.physical_memory_offset);

    // Setup memory and heap
    let mut mapper = unsafe { memory::init(phys_mem_offset) };
    let mut frame_allocator =
        memory::boot_frame_allocator::BootFrameAllocator::new(&boot_info.memory_map);

    memory::heap::init_heap(&mut mapper, &mut frame_allocator).expect("Heap allocation failed");

    // TLS Setup
    let tls_data = boot_info
        .tls_template()
        .expect("TLS section doesn't exist.");
    // We add 8 bytes to total_size to allow storage for %fs value as needed by the x64 ABI
    // See https://stackoverflow.com/a/62240716/2178906
    let tls_ptr = unsafe {
        crate::memory::cpu_local::load_tls_data(
            tls_data.start_addr as *const u8,
            tls_data.file_size as usize,
            tls_data.mem_size as usize + 8,
        )
    };
    // Load FSBase register
    let fs_ptr = ((tls_ptr as *const u8 as u64) + tls_data.mem_size) as *mut u64;
    x86_64::registers::model_specific::FsBase::write(VirtAddr::from_ptr(fs_ptr));
    unsafe {
        *fs_ptr = fs_ptr as u64;
    }

    kernel_info!("tls_ptr is {:x?}", fs_ptr);

    // Setup interrupts
    interrupts::initialize(phys_mem_offset);
}
