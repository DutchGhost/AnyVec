#![feature(untagged_unions)]

use std::mem;

union Inner<A, B, C> {
    a: A,
    b: B,
    c: C,
}

pub struct AnyVec<A, B, C> {
    inner: Vec<Inner<A, B, C>>,
}

pub trait AnyDeref<T> {
    fn anyderef(&self) -> &T;
}

// impl <A, B, C> AnyDeref<Vec<A>> for AnyVec<A, B, C> {
//     fn anyderef(&self) -> &Vec<A> {
//         unsafe { mem::transmute(&self.inner) }
//     }
// }

impl <A, B, C, T> AnyDeref<Vec<T>> for AnyVec<A, B, C> {
    fn anyderef(&self) -> &Vec<T> {
        unsafe { mem::transmute(&self.inner) }
    }
}