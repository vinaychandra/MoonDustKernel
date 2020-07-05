use super::{
    allocator::physical_memory_allocator::IPhysicalMemoryAllocator,
    paging::{IMemoryMapper, MapperPermissions},
};
use crate::arch::globals;
use core::alloc::{GlobalAlloc, Layout};
use linked_list_allocator::LockedHeap;

static STACK_ALLOCATOR: LockedHeap = LockedHeap::empty();

pub struct Stack {
    high_addr: *mut u8,
    size: usize,
}

impl Stack {
    /// Create a new stack with the give size.
    pub fn new_kernel_stack(
        size: usize,
        mapper: &mut dyn IMemoryMapper,
        allocator: &dyn IPhysicalMemoryAllocator,
    ) -> Stack {
        debug_assert!(
            size % globals::PAGE_SIZE == 0,
            "Stack size should be aligned."
        );

        let addr = unsafe {
            STACK_ALLOCATOR.alloc(
                Layout::from_size_align(size + globals::PAGE_SIZE, globals::PAGE_SIZE).unwrap(),
            )
        };

        mapper
            .map_with_alloc(addr, size, MapperPermissions::WRITE, allocator)
            .unwrap();

        unsafe {
            Stack {
                high_addr: addr.offset((size + globals::PAGE_SIZE - 1) as isize),
                size: size + globals::PAGE_SIZE,
            }
        }
    }

    pub fn bsp_kernel_stack(
        mapper: &mut impl IMemoryMapper,
        frame_allocator: &mut dyn IPhysicalMemoryAllocator,
    ) -> Result<Stack, &'static str> {
        info!(target: "new_kernel_stack", "Creating a new kernel bsp stack");
        // We leave one page at the end as a guard page.
        let high_addr = (globals::KERNEL_STACK_BSP
            + globals::KERNEL_STACK_BSP_SIZE
            + globals::PAGE_SIZE) as *mut u8;

        mapper.map_with_alloc(
            (globals::KERNEL_STACK_BSP + globals::PAGE_SIZE) as *const u8,
            globals::KERNEL_STACK_BSP_SIZE,
            MapperPermissions::WRITE,
            frame_allocator,
        )?;

        Ok(Stack {
            high_addr: unsafe { high_addr.offset(-1) },
            size: globals::KERNEL_STACK_BSP_SIZE,
        })
    }

    pub fn get_high_addr(&self) -> *mut u8 {
        self.high_addr
    }
}

impl Drop for Stack {
    fn drop(&mut self) {
        let _s = self.size;
        todo!()
    }
}

pub fn initialize_stack_provider_bsp(
    mapper: &mut dyn IMemoryMapper,
    allocator: &dyn IPhysicalMemoryAllocator,
) {
    let start_addr =
        globals::KERNEL_STACK_BSP + globals::KERNEL_STACK_BSP_SIZE + globals::PAGE_SIZE;
    mapper
        .map_with_alloc(
            start_addr as *const u8,
            globals::KERNEL_STACK_PRE_ALLOCATED,
            MapperPermissions::WRITE,
            allocator,
        )
        .unwrap();
    unsafe {
        STACK_ALLOCATOR
            .lock()
            .init(start_addr, globals::KERNEL_STACK_TOTAL_SIZE);
    }
}
