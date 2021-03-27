use core::sync::atomic::{AtomicUsize, Ordering};

use super::paging::{IMemoryMapper, MapperPermissions};
use crate::{arch::globals, common::align_down};

static NUM_STACKS_ALLOCATED: AtomicUsize = AtomicUsize::new(0);

/// This function creates a guarded kernel stacks. This is used to create kernel stacks
/// for each of the processor core on the machine.
pub fn create_new_kernel_stack(mapper: &mut dyn IMemoryMapper) -> *const () {
    let num_stacks = NUM_STACKS_ALLOCATED.fetch_add(1, Ordering::SeqCst);
    let start_addr = globals::KERNEL_STACK_START + (num_stacks * globals::KERNEL_STACK_GAP);

    mapper
        .map_with_alloc(
            start_addr as *const u8,
            globals::KERNEL_STACK_MAX_SIZE,
            MapperPermissions::WRITE | MapperPermissions::READ,
        )
        .expect("Failed to create kernel stack");

    let stack_end_addr = start_addr + globals::KERNEL_STACK_MAX_SIZE;
    let aligned_stack_end = align_down(stack_end_addr, globals::STACK_ALIGN) - globals::PAGE_SIZE;
    aligned_stack_end as *const ()
}
