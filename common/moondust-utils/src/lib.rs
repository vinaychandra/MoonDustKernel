#![no_std]
#![allow(dead_code)]
#![feature(const_mut_refs, const_fn_fn_ptr_basics)]
#![feature(result_into_ok_or_err)]
#![feature(const_fn)]
#![feature(min_type_alias_impl_trait)]
#![deny(unsafe_op_in_unsafe_fn)]

extern crate alloc;

pub mod bounded_segqueue;
pub mod buddy_system_allocator;
pub mod endpoint;
pub mod executor;
pub mod id_generator;
pub mod interval_tree;
pub mod sync;
