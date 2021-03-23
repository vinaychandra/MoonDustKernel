#![no_std]
#![allow(dead_code)]
#![feature(const_mut_refs, const_fn_fn_ptr_basics)]
#![feature(result_into_ok_or_err)]
#![deny(unsafe_op_in_unsafe_fn)]

extern crate alloc;

pub mod bounded_segqueue;
pub mod buddy_system_allocator;
pub mod executor;
pub mod id_generator;
pub mod interval_tree;
pub mod sync;
