//! This crate wraps around Union types.
//! It provids a way to keep track of the current type of the union,
//! by using the PhantomData struct.

#![feature(untagged_unions)]
#![cfg_attr(feature = "const_id", feature(min_const_fn, const_type_id))]

pub mod index;
pub mod select;
pub mod union;
