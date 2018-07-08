#![feature(const_fn)]
#![feature(const_type_id)]
#![feature(untagged_unions)]
#![feature(allocator_api)]

//#![feature(trace_macros)]
//trace_macros!(true);

extern crate core;
pub mod selectvec;

pub mod select;
pub mod union;

mod macros;
pub use macros::*;
