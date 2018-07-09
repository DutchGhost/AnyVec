use std::marker::PhantomData;
use std::mem;
use std::ptr;

use select::{Select, SelectHandle, Selector, TypeSelect, TypeUnion};

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
    /// Constructs a new, empty `UnionVec<T, U>`.
    /// `T` is the current type of the vector, `U` a tuple of types the vector can change to.
    /// The UnionVector will not allocate until elements are pushed onto it.
    ///
    /// # Examples
    /// ```
    /// extern crate unioncollections;
    ///
    /// use unioncollections::collections::unionvec::UnionVec;
    ///
    /// let unionvec = UnionVec::<u32, (u32, usize)>::new();
    /// ```
    #[inline]
    pub fn new() -> Self {
        Self {
            data: Vec::new(),
            marker: PhantomData,
        }
    }

    /// Constructs a new, empty `UnionVec<T, U>` with the specified capacity.
    /// The UnionVector will be able to hold exactly `capcity` elements with reallocating. If
    /// `capacity` is 0, the union-vector will not allocate.
    ///
    /// It is important to note that altough the returned union-vector has the capacity specified,
    /// the union-vector will have a zero length.
    ///
    /// # Examples
    /// ```
    /// extern crate unioncollections;
    ///
    /// use unioncollections::collections::unionvec::UnionVec;
    ///
    /// let mut v = UnionVec::<String, (String, u32)>::with_capacity(10);
    ///
    /// assert_eq!(v.len(), 0);
    /// ```
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
    pub fn push(&mut self, item: T) {
        let item = SelectHandle::<T, U>::from(item);
        self.data.push(item.into_inner())
    }

    #[inline]
    pub fn pop(&mut self) -> Option<T> {
        self.data.pop().map(|union| unsafe { union.cast::<T>() })
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

        UnionVec {
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
        /*
         * 1) Get the underlying Vec
         * 2) Get the length of the vec
         * 3) Set the vector's length to 0 (panic safety)
         * 4) Make a ptr to the start of the Vec,
         * 5) Loop for i in 0..length,
         * 6) for every i, get the i'th offset from the ptr,
         * 7) make a SelectHandle from the i'th element
         * 8) convert into T,
         * 9) call function
         * 10) make SelectHandle with output,
         * 11) write to the i'th index
         * 12) restore the length
         */

        // 1
        let mut data = self.into_data();

        // 2
        let len = data.len();

        unsafe {
            // 3
            data.set_len(0);

            // 4
            let ptr = data.as_mut_ptr();

            // 5
            for i in 0..len as isize {
                // 6
                let item_ptr: *mut U::Union = ptr.offset(i);

                // 7
                let union_t: SelectHandle<T, U> = SelectHandle::from_inner(ptr::read(item_ptr));

                // 8
                let t: T = union_t.into();

                // 9
                let u = f(t);

                // 10
                let union_u: SelectHandle<<U as Select<S>>::Output, U> =
                    SelectHandle::from_unchecked(u);

                // 11
                ptr::write(item_ptr, union_u.into_inner());
            }

            // 12
            data.set_len(len);
        }

        UnionVec {
            data: data,
            marker: PhantomData,
        }
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

#[cfg(test)]
mod tests {
    use super::*;
    use index::{Type1, Type2};
    #[test]
    fn test_unionvec_change_to() {
        let mut union_vec = UnionVec::<String, (String, u64)>::new();
        union_vec.push(String::from("test"));
        assert_eq!(union_vec.pop(), Some(String::from("test")));

        let mut union_vec = union_vec.change_to::<Type2>();
        union_vec.push(10);

        assert_eq!(union_vec.len(), 1);
        assert_eq!(union_vec.pop(), Some(10));
    }
}
