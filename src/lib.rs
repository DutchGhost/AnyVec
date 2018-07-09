#![feature(const_fn)]
#![feature(const_type_id)]
#![feature(untagged_unions)]
#![feature(allocator_api)]

//#![feature(trace_macros)]
//trace_macros!(true);

extern crate core;
//pub mod selectvec;


//mod macros;
//pub use macros::*;

// Modularized.
pub mod select;
pub mod union;
pub mod index;

pub mod collections;

pub use union::type_id;
