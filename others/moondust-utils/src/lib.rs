#![no_std]
#![allow(dead_code)]
#![feature(unsafe_block_in_unsafe_fn)]
#![feature(const_mut_refs, const_fn_fn_ptr_basics)]
#![deny(unsafe_op_in_unsafe_fn)]

extern crate alloc;

pub mod async_executor;
pub mod bounded_segqueue;
pub mod buddy_system_allocator;
pub mod interval_tree;
