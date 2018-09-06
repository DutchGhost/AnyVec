#![feature(const_fn)]
#![feature(const_type_id)]
#![feature(untagged_unions)]
#![feature(allocator_api)]

//#![feature(trace_macros)]
//trace_macros!(true);

// Modularized.
pub mod index;
pub mod select;
pub mod union;

pub mod collections;

pub use union::type_id;
