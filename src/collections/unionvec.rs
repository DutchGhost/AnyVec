use std::marker::PhantomData;
use std::mem;
use std::ptr;
use std::iter::FromIterator;

use select::{Select, SelectHandle, Selector, TypeSelect, TypeUnion};

use std::alloc::{Alloc, Global, Layout};

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
    pub fn capacity(&self) -> usize {
        self.data.capacity()
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
    /// The returned UnionVec will have the same capacity as the old one.
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

    /// For each element in the collection, the closure is called.
    /// The closure may return any type the Union can turn into. Any closure that does not return
    /// a type the Union can turn into, will result in a compiletime error.
    ///
    /// # Examples
    /// ```
    /// extern crate unioncollections;
    /// use unioncollections::collections::unionvec::UnionVec;
    /// use unioncollections::index::Type2;
    ///
    /// let mut union_vec = UnionVec::<&str, (&str, u64)>::new();
    ///
    /// for s in vec!["10", "20", "30", "40"] {
    ///     union_vec.push(s);
    /// }
    ///
    /// let mut union_vec = union_vec.map::<Type2, _>(|s| s.parse().unwrap());
    ///
    /// assert_eq!(union_vec.len(), 4);
    /// assert_eq!(union_vec.pop(), Some(40));
    ///   ```
    /// # Panic
    ///
    /// When the closure panics, the internal Vector is leaked.
    // @TODO: Optimize this more
    #[inline]
    pub fn map<S: Selector, F>(self, f: F) -> UnionVec<<U as Select<S>>::Output, U>
    where
        U: Select<S>,
        F: Fn(T) -> <U as Select<S>>::Output
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

                // 8 + 9 + 10
                let union_u: SelectHandle<<U as Select<S>>::Output, U> = union_t.map::<S, _>(&f);

                // 11
                ptr::write(item_ptr, union_u.into_inner());

                /*

                // 8
                let t: T = union_t.into();

                // 9
                let u = f(t);

                // 10
                 let union_u: SelectHandle<<U as Select<S>>::Output, U> =
                     SelectHandle::from_unchecked(u);
                */
            }

            // 12
            data.set_len(len);
        }

        UnionVec {
            data,
            marker: PhantomData,
        }
    }

    /// For each element in the collection, the closure is called. The closure returns an Option,
    /// indicating wheter an element should be written back to the collection. All closure outputs
    /// resulting in `Some` will be written, all closure outputs resulting in `None`, will not be
    /// written.
    ///
    /// Note that altough some elements might result in `None`, the capacity of the underlying
    /// collection does not change.
    ///
    /// # Examples
    /// ```
    /// extern crate unioncollections;
    ///
    /// use unioncollections::collections::unionvec::UnionVec;
    /// use unioncollections::index::Type2;
    ///
    /// let mut union_vec = UnionVec::<&str, (&str, u64)>::new();
    /// for s in vec!["10", "20", "30", "40e"] {
    ///     union_vec.push(s);
    /// }
    ///
    /// // Notice the <Type2, _> here, the underscore is the closure, which is infered.
    /// let mut union_vec = union_vec.filter_map::<Type2, _>(|s| s.parse().ok());
    ///
    /// // the last parse failed, so there are only 3 items in the vec.
    ///
    /// assert_eq!(union_vec.len(), 3);
    ///
    /// // the capacity is still 4!
    /// assert_eq!(union_vec.capacity(), 4);
    /// ```
    /// # Panic
    ///
    /// When the closure panics, the internal Vector is leaked.
    //@TODO: make this faster
    // First load up a few items from the Vec, then cast all at once
    #[inline]
    pub fn filter_map<S: Selector, F>(self, f: F) -> UnionVec<<U as Select<S>>::Output, U>
    where
        U: Select<S>,
        F: Fn(T) -> Option<<U as Select<S>>::Output>,
    {
        let mut data = self.into_data();
        let len = data.len();
        let mut nones: usize = 0;

        unsafe {
            data.set_len(0);

            let ptr = data.as_mut_ptr();

            for i in 0..len as isize {
                let read_ptr: *mut U::Union = ptr.offset(i);
                let write_ptr: *mut U::Union = ptr.offset(i - nones as isize);

                let union_t: SelectHandle<T, U> = SelectHandle::from_inner(ptr::read(read_ptr));

                let u = match union_t.filter_map::<S, _>(&f) {
                    Some(item) => item.into_inner(),
                    None => {
                        nones += 1;
                        continue;
                    }
                };

                ptr::write(write_ptr, u);

                // let t = union_t.into();

                // let u = match f(t) {
                //     Some(item) => item,
                //     None => {
                //         nones += 1;

                //         continue;
                //     }
                // };

                // let union_u: SelectHandle<<U as Select<S>>::Output, U> =
                //     SelectHandle::from_unchecked(u);
                // ptr::write(write_ptr, union_u.into_inner());
            }

            data.set_len(len - nones);
        }

        UnionVec {
            data,
            marker: PhantomData,
        }
    }

    #[inline]
    pub fn into_vec(self) -> Result<Vec<T>, AlignError> {
        if mem::align_of::<U::Union>() % mem::align_of::<T>() != 0 {
            return Err(AlignError);
        }

        let mut data = self.into_data();
        let old_cap = data.capacity();

        unsafe {
            let base_read_ptr = data.as_mut_ptr();
            let base_write_ptr = base_read_ptr as *mut T;

            let len = data.len();
            data.set_len(0);

            for i in 0..len as isize {
                let read_ptr: *mut U::Union = base_read_ptr.offset(i);
                let write_ptr: *mut T = base_write_ptr.offset(i);

                let union_t: SelectHandle<T, U> = SelectHandle::from_inner(ptr::read(read_ptr));
                let t = union_t.into();

                ptr::write(write_ptr, t);
            }

            mem::forget(data);

            let old_cap_in_bytes = old_cap * mem::size_of::<U::Union>();
            let new_cap = old_cap_in_bytes / mem::size_of::<T>();

            if old_cap_in_bytes % mem::size_of::<T>() != 0 {
                let nonnull = ptr::NonNull::new(base_read_ptr).unwrap();
                let layout = Layout::array::<U::Union>(old_cap).unwrap();

                let _ = Global.realloc(nonnull.cast(), layout, new_cap * mem::size_of::<T>());
            }

            Ok(Vec::from_raw_parts(base_write_ptr, len, new_cap))
        }
    }

    #[inline]
    pub fn into_vec_map<S: Selector, F>(self, f: F) -> Result<Vec<<U as Select<S>>::Output>, AlignError>
    where
        U: Select<S>,
        F: Fn(T) -> <U as Select<S>>::Output,
    {
        if mem::align_of::<U::Union>() % mem::align_of::<<U as Select<S>>::Output>() != 0 {
            return Err(AlignError);
        }

        let mut data = self.into_data();
        let old_cap = data.capacity();

        unsafe {
            let base_read_ptr = data.as_mut_ptr();
            let base_write_ptr = base_read_ptr as *mut <U as Select<S>>::Output;

            let len = data.len();
            data.set_len(0);

            for i in 0..len as isize {
                let read_ptr: *mut U::Union = base_read_ptr.offset(i);
                let write_ptr: *mut <U as Select<S>>::Output = base_write_ptr.offset(i);

                let union_t: SelectHandle<T, U> = SelectHandle::from_inner(ptr::read(read_ptr));
                let t = union_t.into();

                let u: <U as Select<S>>::Output = f(t);

                ptr::write(write_ptr, u);
            }

            mem::forget(data);

            let old_cap_in_bytes = old_cap * mem::size_of::<U::Union>();
            let new_cap = old_cap_in_bytes / mem::size_of::<<U as Select<S>>::Output>();

            if old_cap_in_bytes % mem::size_of::<T>() != 0 {
                let nonnull = ptr::NonNull::new(base_read_ptr).unwrap();
                let layout = Layout::array::<U::Union>(old_cap).unwrap();

                let _ = Global.realloc(nonnull.cast(), layout, new_cap * mem::size_of::<<U as Select<S>>::Output>());
            }

            Ok(Vec::from_raw_parts(base_write_ptr, len, new_cap))
        }
    }
}

#[derive(Debug)]
pub struct AlignError;

impl <T, U: TypeUnion> FromIterator<T> for UnionVec<T, U> {

    #[inline]
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        let mut mapped_iter = iter.into_iter().map(|item| {
            SelectHandle::<T, U>::from(item).into_inner()
        });

        Self {
            data: Vec::from_iter(mapped_iter),
            marker: PhantomData
        }
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

    #[test]
    fn test_union_vec_map() {
        let mut union_vec = UnionVec::<&str, (&str, u64)>::new();

        for s in vec!["10", "20", "30", "40"] {
            union_vec.push(s);
        }

        let mut union_vec = union_vec.map::<Type2, _>(|s| s.parse().unwrap());

        assert_eq!(union_vec.len(), 4);
        assert_eq!(union_vec.pop(), Some(40));
    }

    #[test]
    fn test_union_vec_filter_map() {
        let mut union_vec = UnionVec::<&str, (&str, u64)>::new();

        for s in vec!["10", "20", "30", "40e"] {
            union_vec.push(s);
        }

        let mut union_vec = union_vec.filter_map::<Type2, _>(|s| s.parse().ok());

        // the last parse failed, so there are only 3 items in the vec.
        assert_eq!(union_vec.len(), 3);

        // the capacity is still 4!
        assert_eq!(union_vec.capacity(), 4);
    }

    #[test]
    fn test_union_vec_into_vec() {
        let mut union_vec = UnionVec::<&str, (i64, &str)>::new();

        for s in vec!["10", "20", "30", "40"] {
            union_vec.push(s);
        }

        let mut union_vec = union_vec.filter_map::<Type1, _>(|s| s.parse().ok());

        let mut v = union_vec.into_vec().unwrap();
        assert_eq!(v.capacity(), 8);
        assert_eq!(v, vec![10, 20, 30, 40]);
    }
}
