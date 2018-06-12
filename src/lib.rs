#![feature(const_fn)]
#![feature(const_type_id)]
#![feature(untagged_unions)]
#![feature(allocator_api)]

//#![feature(trace_macros)]
//trace_macros!(true);

extern crate core;
pub mod selectvec;

mod macros;
pub use macros::*;
