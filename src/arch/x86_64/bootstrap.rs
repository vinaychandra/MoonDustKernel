use super::{globals, memory, PHYSICAL_MEMORY_ALLOCATOR};
use crate::common::memory::paging::{IMemoryMapper, MapperPermissions};
use globals::MEM_MAP_OFFSET_LOCATION;
use log::LevelFilter;
use x86_64::{
    align_down,
    registers::model_specific::EferFlags,
    structures::paging::{page_table::PageTableEntry, OffsetPageTable, PageTableFlags, Translate},
    PhysAddr, VirtAddr,
};

static BSP_STACK: [u8; globals::BSP_STACK_SIZE_BYTES] = [0; globals::BSP_STACK_SIZE_BYTES];

/// Mem map of 512 GiB requires one extra page
const EMPTY_PTE: PageTableEntry = PageTableEntry::new();

#[repr(align(4096))]
struct MemMapEntries([PageTableEntry; 512]);
static mut MEM_MAP_STACK: MemMapEntries = MemMapEntries([EMPTY_PTE; 512]);

pub fn initialize_bootstrap_core() -> ! {
    info!(target: "bootstrap", "Initializing x86_64 architecture");

    // Pages for initial bootstrapping. This acts as an intermediate step.
    // We need this for setting up for the main stacks but the bootloader only provdes 1K in memory.
    let bsp_addr = &BSP_STACK[0] as *const u8 as usize;
    let level_2_addr = align_down(
        (bsp_addr + BSP_STACK.len()) as u64,
        globals::STACK_ALIGN as u64,
    );

    // Switch to level 2.
    unsafe {
        asm!("
        mov rsp, {0}
        mov rbp, {0}
        jmp {1}
        ", in(reg) level_2_addr, sym initialize_bootstrap_core2, options(noreturn));
    }
}

/// Level 2 initializing.
/// This creates a memory map in higher half and then jumps to it.
fn initialize_bootstrap_core2() {
    // Intialize logging
    log::set_logger(&crate::KERNEL_LOGGER)
        .map(|()| log::set_max_level(LevelFilter::Info))
        .expect("Setting logger failed");

    // This enables syscall extensions on x86_64
    {
        let mut efer = x86_64::registers::model_specific::Efer::read();
        efer |= EferFlags::NO_EXECUTE_ENABLE;
        efer |= EferFlags::SYSTEM_CALL_EXTENSIONS;
        unsafe {
            x86_64::registers::model_specific::Efer::write(efer);
        }
    }

    {
        info!(target: "bootstrap", "Create offset mapping");
        // BootBoot maps memory to 0x0.
        let current_page_table = unsafe { memory::active_level_4_table(VirtAddr::new(0x0)) };
        let mut opt = unsafe { OffsetPageTable::new(current_page_table, VirtAddr::new(0)) };

        // Create the page table entries
        // The location where all of memory is mapped to.
        // 0xFFFF_FF00_0000_0000 (entry 510 in P4)
        for i in 0..512usize {
            let mut target_pte = PageTableEntry::new();
            target_pte.set_addr(
                PhysAddr::new(i as u64 * 1024 * 1024 * 1024),
                PageTableFlags::PRESENT
                    | PageTableFlags::WRITABLE
                    | PageTableFlags::HUGE_PAGE
                    | PageTableFlags::GLOBAL,
            );

            unsafe { MEM_MAP_STACK.0[i] = target_pte };
        }
        let pt_index = VirtAddr::new(MEM_MAP_OFFSET_LOCATION).p4_index();

        let pte_ptr = unsafe { &MEM_MAP_STACK.0 as *const [PageTableEntry] as *const u8 };
        let physaddr = opt
            .translate_addr(VirtAddr::from_ptr(pte_ptr))
            .expect("Cannot translate addr of MEM_MAP_STACK");

        let mut entry = PageTableEntry::new();
        entry.set_addr(
            physaddr,
            PageTableFlags::PRESENT | PageTableFlags::WRITABLE | PageTableFlags::GLOBAL,
        );
        opt.level_4_table()[pt_index] = entry;
        x86_64::instructions::tlb::flush_all();
        info!(target: "bootstrap", "Offset mapping complete");
    }

    {
        info!(target: "bootstrap", "Provide free memory to Physical allocator");
        let mut allocator = PHYSICAL_MEMORY_ALLOCATOR.lock();
        let entries = unsafe { crate::bootboot::bootboot.get_mmap_entries() };

        for entry in entries {
            if !entry.is_free() {
                continue;
            }

            let entry_start: usize = entry.ptr();
            let entry_end: usize = entry.end_address() as usize;

            unsafe {
                allocator.add_to_heap(
                    entry_start + (MEM_MAP_OFFSET_LOCATION as usize),
                    entry_end + (MEM_MAP_OFFSET_LOCATION as usize),
                );
            }
        }

        info!(target: "bootstrap", "Provide free memory to Physical allocator complete");
    }

    {
        info!(target: "bootstrap", "Map initial kernel heap");

        let current_page_table =
            unsafe { memory::active_level_4_table(VirtAddr::new(MEM_MAP_OFFSET_LOCATION)) };
        let mut opt = unsafe {
            OffsetPageTable::new(current_page_table, VirtAddr::new(MEM_MAP_OFFSET_LOCATION))
        };

        opt.map_with_alloc(
            globals::KERNEL_HEAP_START as *const u8,
            globals::KERNEL_HEAP_SIZE_INITIAL,
            MapperPermissions::WRITE,
        )
        .expect("Kernel heap init failure");

        info!(target: "bootstrap", "Kernel heap initialized at {:x} with size {} MB and a max of {} MB", 
            globals::KERNEL_HEAP_START,
            globals::KERNEL_HEAP_SIZE_INITIAL / 1024 / 1024,
            globals::KERNEL_HEAP_SIZE_TOTAL / 1024 / 1024);

        info!(target: "bootstrap", "Map initial kernel heap complete");
    }

    {
        info!(target: "bootstrap", "Setup Kernel heap allocator");

        let mut allocator = super::KERNEL_HEAP_ALLOCATOR.lock();
        unsafe {
            allocator.init(globals::KERNEL_HEAP_START, globals::KERNEL_HEAP_SIZE_TOTAL);
        }

        info!(target: "bootstrap", "Setup Kernel heap allocator complete");
    }

    {
        info!(target: "bootstrap", "Initializing TLS");
        memory::cpu_local::initialize_tls();
        info!(target: "bootstrap", "TLS Initialized");
    }

    {
        info!(target: "bootstrap", "Initialize GDT");
        super::gdt::initialize_gdt();
        info!(target: "bootstrap", "GDT ready");
    }
}