use super::{
    allocator::physical_memory_allocator,
    allocator::physical_memory_allocator::IPhysicalMemoryAllocator,
    paging::{IMemoryMapper, MapperPermissions},
};
use crate::{arch::globals, common::align_down};
use core::alloc::{GlobalAlloc, Layout};
use linked_list_allocator::LockedHeap;

static STACK_ALLOCATOR: LockedHeap = LockedHeap::empty();

#[derive(Debug)]
pub struct Stack {
    high_addr: *mut u8,
    size: usize,

    frame_pointer: *mut u8,
    stack_pointer: *mut u8,
}

impl Stack {
    pub const fn empty() -> Stack {
        Stack {
            high_addr: core::ptr::null_mut(),
            size: 0,
            frame_pointer: core::ptr::null_mut(),
            stack_pointer: core::ptr::null_mut(),
        }
    }

    /// Create a new stack with the given size.
    pub fn new_kernel_stack(size: usize) -> Stack {
        debug_assert!(
            size % globals::PAGE_SIZE == 0,
            "Stack size should be aligned."
        );

        let addr = unsafe {
            STACK_ALLOCATOR.alloc(
                Layout::from_size_align(size + globals::PAGE_SIZE, globals::PAGE_SIZE).unwrap(),
            )
        };

        // we ignore the fist page so that it throws a page fault
        // mapper
        //     .map_with_alloc(
        //         (addr as usize + globals::PAGE_SIZE) as *mut u8,
        //         size,
        //         MapperPermissions::WRITE,
        //         allocator,
        //     )
        //     .unwrap();

        let high_addr = align_down(
            addr as u64 + size as u64 + globals::PAGE_SIZE as u64,
            globals::STACK_ALIGN as u64,
        ) as *mut u8;

        Stack {
            high_addr,
            size,
            frame_pointer: high_addr,
            stack_pointer: high_addr,
        }
    }

    /// Create a new user stack with the given size.
    pub fn new_user_stack(size: usize, mapper: &mut dyn IMemoryMapper) -> Stack {
        debug_assert!(
            size % globals::PAGE_SIZE == 0,
            "Stack size should be aligned."
        );

        let allocator = physical_memory_allocator::get_physical_memory_allocator();

        let addr = (globals::USER_STACK_END - size + 1) as *mut u8;
        mapper
            .map_with_alloc(
                addr,
                size,
                MapperPermissions::READ | MapperPermissions::RING_3 | MapperPermissions::WRITE,
                allocator,
            )
            .unwrap();

        let high_addr =
            align_down(globals::USER_STACK_END as u64, globals::STACK_ALIGN as u64) as *mut u8;
        Stack {
            high_addr: high_addr,
            size: size + globals::PAGE_SIZE,
            frame_pointer: high_addr,
            stack_pointer: high_addr,
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

        let high_addr = align_down(high_addr as u64, globals::STACK_ALIGN as u64) as *mut u8;
        let result = Ok(Stack {
            high_addr,
            size: globals::KERNEL_STACK_BSP_SIZE,
            frame_pointer: high_addr,
            stack_pointer: high_addr,
        });
        info!(target: "new_kernel_stack", "Kernel BSP Stack: {:?}", result);
        result
    }

    pub fn get_high_addr(&self) -> *mut u8 {
        self.high_addr
    }

    /// return framepointer, stack pointer.
    #[inline(always)]
    pub fn get_stack_pointers(&self) -> (*mut u8, *mut u8) {
        (self.frame_pointer, self.stack_pointer)
    }

    /// return framepointer, stack pointer.
    #[inline(always)]
    pub fn set_stack_pointers(&mut self, fp: *mut u8, sp: *mut u8) {
        self.frame_pointer = fp;
        self.stack_pointer = sp;
    }
}

impl Drop for Stack {
    fn drop(&mut self) {
        let _s = self.size;
        todo!()
    }
}

/// Initialize stack provider on BSP.
pub fn initialize_stack_provider_bsp(mapper: &mut dyn IMemoryMapper) {
    let allocator = super::allocator::physical_memory_allocator::get_physical_memory_allocator();
    let start_addr =
        globals::KERNEL_STACK_BSP + globals::KERNEL_STACK_BSP_SIZE + 2 * globals::PAGE_SIZE;
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
