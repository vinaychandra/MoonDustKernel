use super::{
    allocator::physical_memory_allocator::IPhysicalMemoryAllocator,
    paging::{IMemoryMapper, MapperPermissions},
};
use crate::arch::globals;
use core::{
    alloc::{GlobalAlloc, Layout},
    sync::atomic::{AtomicPtr, Ordering},
};
use linked_list_allocator::LockedHeap;

static STACK_ALLOCATOR: LockedHeap = LockedHeap::empty();

pub struct Stack {
    high_addr: *mut u8,
    size: usize,

    frame_pointer: AtomicPtr<u8>,
    stack_pointer: AtomicPtr<u8>,
}

impl Stack {
    /// Create a new stack with the given size.
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
            let high_addr = addr.offset((size + globals::PAGE_SIZE - 1) as isize);

            Stack {
                high_addr: high_addr,
                size: size + globals::PAGE_SIZE,
                frame_pointer: AtomicPtr::new(high_addr),
                stack_pointer: AtomicPtr::new(high_addr),
            }
        }
    }

    /// Create a new user stack with the given size.
    pub fn new_user_stack(
        size: usize,
        mapper: &mut dyn IMemoryMapper,
        allocator: &dyn IPhysicalMemoryAllocator,
    ) -> Stack {
        debug_assert!(
            size % globals::PAGE_SIZE == 0,
            "Stack size should be aligned."
        );

        let addr = globals::USER_STACK_END - size + 1;
        mapper
            .map_with_alloc(
                addr as *mut u8,
                size,
                MapperPermissions::READ | MapperPermissions::RING_3 | MapperPermissions::WRITE,
                allocator,
            )
            .unwrap();

        let high_addr = globals::USER_STACK_END as *mut u8;
        Stack {
            high_addr: high_addr,
            size: size + globals::PAGE_SIZE,
            frame_pointer: AtomicPtr::new(high_addr),
            stack_pointer: AtomicPtr::new(high_addr),
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

        let high_addr = unsafe { high_addr.offset(-1) };
        Ok(Stack {
            high_addr: high_addr,
            size: globals::KERNEL_STACK_BSP_SIZE,
            frame_pointer: AtomicPtr::new(high_addr),
            stack_pointer: AtomicPtr::new(high_addr),
        })
    }

    pub fn get_high_addr(&self) -> *mut u8 {
        self.high_addr
    }

    // return framepointer, stack pointer.
    pub fn get_stack_pointers(&self) -> (*mut u8, *mut u8) {
        (
            self.frame_pointer.load(Ordering::SeqCst),
            self.stack_pointer.load(Ordering::SeqCst),
        )
    }

    pub fn set_stack_pointers(&self, frame_pointer: *mut u8, stack_pointer: *mut u8) {
        self.frame_pointer.store(frame_pointer, Ordering::SeqCst);
        self.stack_pointer.store(stack_pointer, Ordering::SeqCst);
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
