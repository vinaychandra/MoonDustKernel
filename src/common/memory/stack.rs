use super::{
    allocator::physical_memory_allocator::IPhysicalMemoryAllocator,
    paging::{MapperPermissions, MemoryMapper},
};
use crate::arch::globals;

pub struct Stack {
    high_addr: *mut u8,
}

impl Stack {
    pub fn bsp_kernel_stack(
        mapper: &mut impl MemoryMapper,
        frame_allocator: &mut impl IPhysicalMemoryAllocator,
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
        })
    }

    pub fn get_high_addr(&self) -> *mut u8 {
        self.high_addr
    }
}
