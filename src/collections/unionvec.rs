use std::marker::PhantomData;
use std::mem;

use select::{Select, Selector, TypeUnion};

/// A UnionVec can be used to hold multiple datatypes, but only one at a time.
/// It's possible to change between types, but only for all items, and not individually per item.
///
/// Changing between types can be done with [`UnionVec::change_to`], [`UnionVec::map`] and
/// [`UnionVec::into_vec`]. It's also possible to discard values, with [`UnionVec::filter_map`]
#[derive(Ord, PartialOrd, Eq, PartialEq, Clone, Default, Hash)]
pub struct UnionVec<T: 'static, U: TypeUnion> {
    data: Vec<U::Union>,
    marker: PhantomData<T>,
}

impl<T: 'static, U: TypeUnion> UnionVec<T, U> {
    #[inline]
    pub fn new() -> Self {
        Self {
            data: Vec::new(),
            makrker: PhantomData,
        }
    }

    #[inline]
    pub fn with_capacity(n: usize) -> Self {
        Self {
            data: Vec::with_capacity(n),
            marker: PhantomData,
        }
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.data.len()
    }

    #[inline]
    pub fn into_data(self) -> Vec<U::Union> {
        let data = unsafe { ptr::read(&self.data) };
        mem::forget(self);
        data
    }

    /// Clears the underlying Vec, and returns a new [`UnionVec`].
    /// The returned UnionVec will have the same capacity as the old one had.
    #[inline]
    pub fn change_to<S>(mut self) -> UnionVec<<U as Select<S>>::Output, U>
    where
        S: Selector,
        U: Select<S>,
    {
        self.data.clear();

        Self {
            data: self.into_data(),
            marker: PhantomData,
        }
    }

    #[inline]
    pub fn map<S: Selector, F>(self, f: F) -> UnionVec<<U as Select<S>>::Output, U>
    where
        U: Select<S>,
        F: Fn(T) -> <U as Select<S>>::Output,
    {
        unimplemented!()
    }

    #[inline]
    pub fn filter_map<S: Selector, F>(self, f: F) -> UnionVec<<U as Select<S>>::Output, U>
    where
        U: Select<S>,
        F: Fn(T) -> Option<<U as Select<S>>::Output>,
    {
        unimplemented!()
    }

    #[inline]
    pub fn into_vec(self) -> Vec<T> {
        unimplemented!()
    }
}
