use std::ptr;
use std::mem;
use std::fmt;
use std::iter;
use std::any::TypeId;
use std::marker::PhantomData;
use std::convert::{AsRef, AsMut};

use core::alloc::{Layout, Alloc};
use std::alloc::Global;

/// Returns the TypeId of `T`.
pub const fn type_id<T: 'static>() -> TypeId {
    TypeId::of::<T>()
}

pub unsafe trait TypeSelect<U: TypeUnion> : Sized {
    
    /// Casts `self` to `T`.
    /// This should only be used in context with types implementing [`TypeUnion`],
    /// to safely cast the union into its current held datatype.
    /// 
    /// # Panic
    /// 
    /// When a cast to a datatype happens, that is not part of the Union,
    /// this function will panic.
    #[inline]
    unsafe fn cast<T>(self) -> T where T: 'static {
        debug_assert!(U::contains::<T>());
        let mut s = mem::uninitialized();
        ptr::write(&mut s as *mut _ as *mut Self, self);
        s
    }

    /// Casts `self` into a [`SelectItem`].
    #[inline]
    unsafe fn select<S>(self) -> SelectItem<<U as Select<S>>::Output, U> 
        where S: Selector, U: Select<S>
    {
        self.cast()
    }
}

/// This trait is used to check at runtime whether any type `T` equals one of a sequence of other types.
/// Part of this check can happen during compilation, since we know the types of `T` and the sequence at compile time,
/// the only part at runtime is the comparison.
pub trait TypeUnion: Sized + 'static {
    type Union: TypeSelect<Self>;

    /// Returns `true` if `T` is one of a sequence of other types.
    fn contains<T: 'static>() -> bool;

}

/// Helper trait to index into a tuple of Generics.
pub trait Selector {}

/// Helper trait to 'select' a generic type out of a tuple of Generics.
pub trait Select<S: Selector> {

    /// The current selected type.
    type Output: 'static;
}

/// Struct to safely convert from a [`TypeUnion`] to `T`, and vice versa.
/// 
/// # Examples
/// ```
/// use selectvec::{A, selectvec::{SelectItem, Selector}};
/// 
/// let mut item: SelectItem<u32, (u32, String, ())> = SelectItem::from::<A>(10);
/// ```
pub struct SelectItem<T, D: TypeUnion>
{
    data: D::Union,
    marker: PhantomData<T>,
}

impl<T, D> SelectItem<T, D>
where
    T: 'static,
    D: TypeUnion,
{    
    /// Creates a new `SelectItem<T, D>` from a `T`
    #[inline]
    pub fn from<S>(t: T) -> SelectItem<T, D>
        where S: Selector, D: Select<S, Output=T>,
    {   
        unsafe { Self::from_unchecked(t) }
    }

    #[inline]
    pub unsafe fn from_unchecked(t: T) -> Self
    {
        let mut s = mem::uninitialized();
        ptr::write(&mut s as *mut _ as *mut T, t);
        s
    }

    /// Converts `self` back into `T`.
    #[inline]
    pub fn into(mut self) -> T {
        unsafe {
            let t = ptr::read(&mut self as *mut _ as *mut T);
            mem::forget(self);
            t
        }
    }

    /// Returns the Union contained.
    #[inline]
    pub fn into_inner(self) -> D::Union {
        self.data
    }

    /// Creates a new `SelectItem` from A Union.
    #[inline]
    pub unsafe fn from_inner(data: D::Union) -> Self {
        SelectItem {
            data,
            marker: PhantomData,
        }
    }
}

impl<T, D> AsRef<T> for SelectItem<T, D> where D: TypeUnion {
    #[inline]
    fn as_ref(&self) -> &T {
        unsafe { mem::transmute(&self.data) }
    }
}

impl<T, D> AsMut<T> for SelectItem<T, D> where D: TypeUnion {
    #[inline]
    fn as_mut(&mut self) -> &mut T {
        unsafe { mem::transmute(&mut self.data) }
    }
}

impl<T, D> fmt::Debug for SelectItem<T, D>
where
    D: TypeUnion, T: fmt::Debug
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.as_ref().fmt(f)
    }
}

/// A slice that can change the type of the buffer in a controlled way.
pub struct SelectSlice<'a, T, D>
where
    T: 'static,
    D: 'a + TypeUnion,
{
    data: &'a mut [D::Union],
    marker: PhantomData<T>
}

impl <'a, T, D> SelectSlice<'a, T, D>
where
    T: 'static,
    D: 'a + TypeUnion,
{
    
    #[inline]
    pub const fn current_type(&self) -> TypeId {
        type_id::<T>()
    }

    /// Returns a by-reference Iterator over the items contained in the Vector.
    #[inline]
    pub fn iter(&'a self) -> impl DoubleEndedIterator<Item = &'a T> {
        self.data.iter().map(|item| unsafe { mem::transmute(item) })
    }

    /// Returns a by-mutable-reference Iterator over the items contained in the Vector.
    /// This allows for mutation.
    pub fn iter_mut(&'a mut self) -> impl DoubleEndedIterator<Item = &'a mut T> {
        self.data.iter_mut().map(|item| unsafe { mem::transmute(item) })
    }

    #[inline]
    pub fn change_type<S>(self) -> SelectSlice<'a, <D as Select<S>>::Output, D>
    where
        S: Selector, D: Select<S>
    {
        SelectSlice {
            data: self.data,
            marker: PhantomData,
        }
    }

    #[inline]
    pub fn map<S: Selector, F>(self, f: F) -> SelectSlice<'a, <D as Select<S>>::Output, D>
    where
        D: Select<S>,
        F: Fn(T) -> <D as Select<S>>::Output
    {
        let data = self.data;
        unsafe {
            let ptr = data.as_mut_ptr();
            let len = data.len();
            
            for i in 0..len as isize {
                let item_ptr: *mut D::Union = ptr.offset(i);
                let any_t: SelectItem<T, D> = SelectItem::from_inner(ptr::read(item_ptr));
                let t: T = any_t.into();
                let u = f(t);
                let any_u: SelectItem<<D as Select<S>>::Output, D> = SelectItem::from_unchecked(u);
                ptr::write(item_ptr, any_u.into_inner());
            }
        }

        SelectSlice {data, marker: PhantomData}
    }
}

/// A Vector that can hold multiple data-types, and switch to those data-types, without losing its allocated space.
#[derive(Ord, PartialOrd, Eq, PartialEq, Clone)]
pub struct SelectVec<T, D>
where
    T: 'static,
    D: TypeUnion
{
    data: Vec<D::Union>,
    marker: PhantomData<T>
}

impl<T, D> SelectVec<T, D>
where
    T: 'static,
    D: TypeUnion,
{
    
    /// Creates a new empty `SelectVec<T, D>`.
    #[inline]
    pub fn new() -> Self {
        SelectVec {
            data: Vec::new(),
            marker: PhantomData,
        }
    }

    /// Creates a new empty `SelectVec<T, D>`.
    /// The Vector will be able to hold exactly `capacity` items without reallocating.
    #[inline]
    pub fn with_capacity(capacity: usize) -> Self {
        SelectVec {
            data: Vec::with_capacity(capacity),
            marker: PhantomData,
        }
    }
    
    /// Returns the type of the current selected type.
    pub const fn current_type(&self) -> TypeId {
        type_id::<T>()
    }

    /// Returns the length of the underlying Vector.
    #[inline]
    pub fn len(&self) -> usize {
        self.data.len()
    }

    /// Returns the capacity of the underlying Vector.
    #[inline]
    pub fn capacity(&self) -> usize {
        self.data.capacity()
    }
    /// Pushes an item onto the underlying Vector
    #[inline]
    pub fn push(&mut self, item: T)
    {
        let item = unsafe { SelectItem::<T, D>::from_unchecked(item) };
        self.data.push(item.into_inner());
    }
    
    /// Pops the last pushed item from the underlying Vector.
    #[inline]
    pub fn pop(&mut self) -> Option<T>
    {
        self.data.pop().map(|i| unsafe {
            i.cast::<T>()
        })
    }

    /// Consumes self, returning the underlying Vector.
    #[inline]
    pub fn into_data(self) -> Vec<D::Union> {
        let data = unsafe { ptr::read(&self.data) };
        mem::forget(self);
        data
    }

    /// Creates a `SelectSlice`, a slice into the buffer of this struct.
    /// SelectSlice can also map, and change the data-type of the buffer.
    #[inline]
    pub fn as_select_slice(&mut self) -> SelectSlice<T, D> {
        SelectSlice {
            data: self.data.as_mut(),
            marker: PhantomData,
        }
    }

    /// Returns a by-reference Iterator over the items contained in the Vector.
    #[inline]
    pub fn iter<'a>(&'a self) -> impl DoubleEndedIterator<Item = &'a T> {
        self.data.iter().map(|item| unsafe { mem::transmute(item) })
    }

    /// Returns a by-mutable-reference Iterator over the items contained in the Vector.
    /// This allows for mutation.
    pub fn iter_mut<'a>(&'a mut self) -> impl DoubleEndedIterator<Item = &'a mut T> {
        self.data.iter_mut().map(|item| unsafe { mem::transmute(item) })
    }

    // /// Returns a by-value Iterator over the items contained in the Vector.
    // #[inline]
    // pub fn into_iter(self) -> impl Iterator<Item = T>
    // {
    //     self.into_data().into_iter().map(|i| unsafe {
    //         let item = SelectItem::<T, D>::from_inner(i);
    //         item.into()
    //     })
    // }

    /// Returns a draining Iterator, consuming the range of items specified.
    #[inline]
    pub fn drain<'a, R>(&'a mut self, r: R) -> impl DoubleEndedIterator<Item = T> + 'a
    where
        R: ::std::ops::RangeBounds<usize>
    {
        self.data.drain(r).map(move |i| unsafe {
            let item = SelectItem::<T, D>::from_inner(i);
            item.into()
        })
    }

    /// Returns a new instance of `SelectVec<U, A, B, C>` by clearing the current vector, leaving the allocated space untouched.
    /// It returns a new `SelectVec<U, A, B, C>`, that can now hold a different data-type.
    /// # Examples
    /// ```
    /// use selectvec::{C, selectvec::SelectVec};
    /// let mut vec = SelectVec::<char, (char, u8, String)>::new();
    ///
    /// vec.push('a');
    ///
    /// let mut changed = vec.change_type::<C>();
    ///
    /// changed.push(String::from("foo"));
    ///
    /// assert_eq!(changed.pop(), Some(String::from("foo")));
    /// ```
    ///
    /// # Panic
    /// Trying to change to a datatype that is not specified at creation, is not allowed, and will result in a panic!():
    ///
    /// ```comptime_fail
    /// 
    /// use selectvec::{C, selectvec::SelectVec};
    /// let mut vec = SelectVec::<char, (char, u8, String)>::new();
    ///
    /// vec.push('a');car
    ///
    /// let mut changed = vec.change_type::<C>();
    /// changed.push(10);
    /// assert_eq!(changed.pop(), Some(10));
    ///
    /// ```
    #[inline]
    pub fn change_type<S>(mut self) -> SelectVec<<D as Select<S>>::Output, D>
    where
        S: Selector, D: Select<S>
    {
        self.data.clear();

        SelectVec {
            data: self.into_data(),
            marker: PhantomData,
        }
    }

    /// With this function, you can change the type the Vector holds in place.
    /// This does not allocate new space.
    /// Notice that this function has to take a Generic parameter:
    /// 
    /// - 'A' will change the type to the first of the types provided at creation of the SelectVec.
    /// 
    /// - 'B' will change the type to the second of the types provided at creation of the SelectVec.
    /// 
    /// - 'C' will change the type to the third of the types provided at creation of the SelectVec.
    /// 
    /// - etc, etc.
    /// 
    /// If the type the closure returns does not match with the new selected type, you will get a compiler error.
    /// 
    /// # Examples
    /// ```
    /// use selectvec::{B, selectvec::SelectVec};
    ///
    /// let mut vec = SelectVec::<&str, (&str, Result<u32, ()>, u32)>::new();
    ///
    /// vec.push("10");
    /// vec.push("20");
    /// vec.push("30");
    /// vec.push("40");
    ///
    /// let mut changed = vec.map::<B, _>(|s| s.parse::<u32>().map_err(|_| ()) );
    ///
    /// {
    ///     let mut iter = changed.iter();
    ///
    ///     assert_eq!(iter.next(), Some(&Ok(10)));
    ///     assert_eq!(iter.next(), Some(&Ok(20)));
    ///     assert_eq!(iter.next(), Some(&Ok(30)));
    ///     assert_eq!(iter.next(), Some(&Ok(40)));
    /// }
    ///
    /// ```
    /// 
    /// # Safety
    /// If the closure panics, the internal vector is leaked.
    /// 
    #[inline]
    pub fn map<S: Selector, F>(self, f: F) -> SelectVec<<D as Select<S>>::Output, D>
    where
        D: Select<S>,
        F: Fn(T) -> <D as Select<S>>::Output
    {
        let mut data = self.into_data();

        unsafe {
            let ptr = data.as_mut_ptr();
            let len = data.len();
            data.set_len(0);

            for i in 0..len as isize {
                let item_ptr: *mut D::Union = ptr.offset(i);
                let any_t: SelectItem<T, D> = SelectItem::from_inner(ptr::read(item_ptr));
                let t: T = any_t.into();
                let u = f(t);
                let any_u: SelectItem<<D as Select<S>>::Output, D> = SelectItem::from_unchecked(u);
                ptr::write(item_ptr, any_u.into_inner());
            }

            data.set_len(len);
        }

        SelectVec {data, marker: PhantomData}
    }

    /// This function has exactly the same context as [`SelectVec::map`], however notice that the type the closure returns is an Option.
    /// When the closure returns None, the item does not get written to the Vector, therefore truncating the Vector.
    /// 
    /// The allocated space is not touched.
    /// 
    /// # Examples
    /// ```
    /// use selectvec::{B, selectvec::SelectVec};
    /// 
    /// let mut vec = (0..10).collect::<SelectVec<u32, (u32, String, ())>>();
    /// 
    /// let mut stringvec = vec.maybe_map::<B, _>(|n| {
    ///     if n & 1 == 0 {
    ///         Some(n.to_string())
    ///     } else {
    ///         None
    ///     }
    /// });
    /// 
    /// let mut iter = stringvec.into_iter();
    /// 
    /// assert_eq!(iter.next(), Some(String::from("0")));
    /// assert_eq!(iter.next(), Some(String::from("2")));
    /// assert_eq!(iter.next(), Some(String::from("4")));
    /// assert_eq!(iter.next(), Some(String::from("6")));
    /// assert_eq!(iter.next(), Some(String::from("8")));
    /// assert_eq!(iter.next(), None);
    /// ```
    #[inline]
    pub fn maybe_map<S: Selector, F>(self, f: F) -> SelectVec<<D as Select<S>>::Output, D>
    where
        D: Select<S>,
        F: Fn(T) -> Option<<D as Select<S>>::Output>
    {
        let mut data = self.into_data();
        let mut failures: usize = 0;

        unsafe {
            let ptr = data.as_mut_ptr();
            let len = data.len();
            
            data.set_len(0);

            for i in 0..len as isize {
                let read_ptr: *mut D::Union = ptr.offset(i);
                let write_ptr: *mut D::Union = ptr.offset(i - failures as isize);
                let any_t: SelectItem<T, D> = SelectItem::from_inner(ptr::read(read_ptr));
                let t: T = any_t.into();
                let u = match f(t) {
                    Some(item) => item,
                    
                    //on None, increment failures.
                    None => {
                        failures += 1;
                        continue;
                    }
                };
                let any_u: SelectItem<<D as Select<S>>::Output, D> = SelectItem::from_unchecked(u);
                ptr::write(write_ptr, any_u.into_inner());
            }

            data.set_len(len - failures);
        }

        SelectVec {data, marker: PhantomData}
    }

    /// Converts the SelectVec into a regular Vector, re-using the allocation.
    /// Note that this can only be done when the Alignment of the Union `D`,
    /// is equal to the alignment of the current held type.
    /// 
    /// If you want to change the data-type before converthing into a Vector, use [`SelectVec::try_to_vec_map()`]
    /// 
    /// # Examples
    /// ```
    /// use selectvec::{B, selectvec::SelectVec};
    /// 
    /// let mut v = (0..5).collect::<SelectVec<u32, (u32, String, ())>>();
    /// 
    /// let string_svec = v.map::<B, _>(|n| n.to_string());
    /// 
    /// let ss = string_svec.try_to_vec();
    /// 
    /// let mut iter = ss.into_iter();
    /// 
    /// assert_eq!(iter.next(), Some(String::from("0")));
    /// assert_eq!(iter.next(), Some(String::from("1")));
    /// assert_eq!(iter.next(), Some(String::from("2")));
    /// assert_eq!(iter.next(), Some(String::from("3")));
    /// assert_eq!(iter.next(), Some(String::from("4")));
    /// assert_eq!(iter.next(), None);
    /// ```
    #[inline]
    pub fn try_to_vec(self) -> Vec<T>
    {
        if mem::align_of::<D::Union>() % mem::align_of::<T>() != 0 {
            panic!("Can not convert a Vector with items that have an alignment of {},
                    into a Vector with items that have an alignment of {}.",
                     mem::align_of::<D::Union>(), mem::align_of::<T>()
            );
        }

        let mut data = self.into_data();
        let old_cap = data.capacity();

        unsafe {
            
            let base_read_ptr = data.as_mut_ptr();
            let base_write_ptr = base_read_ptr as *mut T;

            let len = data.len();
            data.set_len(0);
            
            for i in 0..len as isize {
                let read_ptr: *mut D::Union = base_read_ptr.offset(i);
                let write_ptr: *mut T = base_write_ptr.offset(i);

                let any_t: SelectItem<T, D> = SelectItem::from_inner(ptr::read(read_ptr));
                let t: T = any_t.into();

                ptr::write(write_ptr, t);
            }

            //DONT DROP DATA, WE CREATE A NEW VEC FROM IT USING A PTR. JUST FORGET ABOUT IT.
            mem::forget(data);
            
            //calculate old capacity in bytes,
            let old_cap_in_bytes = old_cap * mem::size_of::<D::Union>();
            let new_capacity = old_cap_in_bytes / mem::size_of::<T>();

            // realloc            
            if old_cap_in_bytes % mem::size_of::<T>() != 0 {
                
                let nonnull = ptr::NonNull::new(base_read_ptr).unwrap();
                let layout = Layout::array::<D::Union>(old_cap).unwrap();

                let _ = Global.realloc(nonnull.as_opaque(), layout, new_capacity * mem::size_of::<T>());
            }

            Vec::from_raw_parts(base_write_ptr, len, new_capacity)
        }
    }

    /// This function has the same principle as [`SelectVec::try_to_vec()`].
    /// The only difference is that the closure is called for each item, before it gets written back,
    /// therefore saving a call to [`SelectVec::map()`] when you want to change the type in place, but also want a Vector back.
    /// 
    /// # Examples
    /// ```
    /// use selectvec::{B, selectvec::SelectVec};
    /// 
    /// let mut v = (0..5).collect::<SelectVec<u32, (u32, String, ())>>();
    /// 
    /// let ss = v.try_to_vec_map::<B, _>(|n| n.to_string());
    /// 
    /// let mut iter = ss.into_iter();
    /// 
    /// assert_eq!(iter.next(), Some(String::from("0")));
    /// assert_eq!(iter.next(), Some(String::from("1")));
    /// assert_eq!(iter.next(), Some(String::from("2")));
    /// assert_eq!(iter.next(), Some(String::from("3")));
    /// assert_eq!(iter.next(), Some(String::from("4")));
    /// assert_eq!(iter.next(), None);
    /// ```
    /// 
    /// # Safety
    /// If the closure panics, the internal Vector is leaked. 
    #[inline]
    pub fn try_to_vec_map<S: Selector, F>(self, f: F) -> Vec<<D as Select<S>>::Output>
    where
        D: Select<S>,
        F: Fn(T) -> <D as Select<S>>::Output,
    {
        if mem::align_of::<D::Union>() % mem::align_of::<<D as Select<S>>::Output>() != 0 {
            panic!("Can not convert a Vector with items that have an alignment of {},
                    into a Vector with items that have an alignment of {}.",
                     mem::align_of::<D::Union>(), mem::align_of::<<D as Select<S>>::Output>()
            );
        }

        let mut data = self.into_data();
        let old_cap = data.capacity();

        unsafe {
            
            let base_read_ptr = data.as_mut_ptr();
            let base_write_ptr = base_read_ptr as *mut <D as Select<S>>::Output;

            let len = data.len();
            data.set_len(0);
            
            for i in 0..len as isize {
                let read_ptr: *mut D::Union = base_read_ptr.offset(i);
                let write_ptr: *mut <D as Select<S>>::Output = base_write_ptr.offset(i);

                let any_t: SelectItem<T, D> = SelectItem::from_inner(ptr::read(read_ptr));
                let t: T = any_t.into();
                let u: <D as Select<S>>::Output = f(t);

                ptr::write(write_ptr, u);
            }

            //DONT DROP DATA, WE CREATE A NEW VEC FROM IT USING A PTR. JUST FORGET ABOUT IT.
            mem::forget(data);
            
            //calculate old capacity in bytes,
            let old_cap_in_bytes = old_cap * mem::size_of::<D::Union>();
            let new_capacity = old_cap_in_bytes / mem::size_of::<<D as Select<S>>::Output>();

            // realloc            
            if old_cap_in_bytes % mem::size_of::<<D as Select<S>>::Output>() != 0 {
                
                let nonnull = ptr::NonNull::new(base_read_ptr).unwrap();
                let layout = Layout::array::<D::Union>(old_cap).unwrap();

                let _ = Global.realloc(nonnull.as_opaque(), layout, new_capacity * mem::size_of::<<D as Select<S>>::Output>());
            }

            Vec::from_raw_parts(base_write_ptr, len, new_capacity)
        }
    }
}

impl <T, D> Drop for SelectVec<T, D>
where
    D: TypeUnion
{
    fn drop(&mut self)
    {
        for _ in self.drain(..) {}
    }
}

impl <T, D> iter::FromIterator<T> for SelectVec<T, D>
where
    T: 'static,
    D: TypeUnion,
{
    #[inline]
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        let data = iter.into_iter().map(|item| unsafe {
            SelectItem::<T, D>::from_unchecked(item).into_inner()
        }).collect();

        SelectVec {
            data,
            marker: PhantomData
        }
    }
}

impl <T: 'static, D: TypeUnion> IntoIterator for SelectVec<T, D> {
    type Item = T;
    type IntoIter = IntoIter<T, D>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        IntoIter {
            iter: self.into_data().into_iter(),
            marker: PhantomData,
        }
    }
}

pub struct Iter <'a, T, D>
where
    T: 'a,
    D: 'a,
{
    iter: ::std::slice::Iter<'a, D>,
    marker: PhantomData<&'a T>
}

impl <'a, T, D> Iterator for Iter<'a, T, D>
where
    T: 'a,
    D: 'a,
{
    type Item = &'a T;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next().map(|item| unsafe {
            mem::transmute(item)
        })
    }
}

impl <'a, T, D> DoubleEndedIterator for Iter <'a, T, D>
where
    T: 'a,
    D: 'a,
{
    #[inline]
    fn next_back(&mut self) -> Option<Self::Item> {
        self.iter.next_back().map(|item| unsafe {
            mem::transmute(item)
        })
    }
}

pub struct IterMut<'a, T, D>
where
    T: 'a,
    D: 'a,
{
    iter: ::std::slice::IterMut<'a, D>,
    marker: PhantomData<&'a mut T>
}

impl <'a, T, D> Iterator for IterMut<'a, T, D>
where
    T: 'a,
    D: 'a,
{
    type Item = &'a mut T;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next().map(|item| unsafe {
            mem::transmute(item)
        })
    }
}

impl <'a, T, D> DoubleEndedIterator for IterMut<'a, T, D>
where
    T: 'a,
    D: 'a
{
    #[inline]
    fn next_back(&mut self) -> Option<Self::Item> {
        self.iter.next_back().map(|item| unsafe {
            mem::transmute(item)
        })
    }
}

pub struct IntoIter<T: 'static, D: TypeUnion> {
    iter: ::std::vec::IntoIter<D::Union>,
    marker: PhantomData<T>
}

impl <T: 'static, D: TypeUnion> Iterator for IntoIter<T, D>
{
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next().map(|i| unsafe {
            let item = SelectItem::<T, D>::from_inner(i);
            item.into()
        })
    }
}

impl <T: 'static, D: TypeUnion> DoubleEndedIterator for IntoIter<T, D> {
    fn next_back(&mut self) -> Option<Self::Item> {
        self.iter.next_back().map(|i| unsafe {
            let item = SelectItem::<T, D>::from_inner(i);
            item.into()
        })
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_rev_iter() {
        use super::*;

        let mut vec = (0..10).collect::<SelectVec<u32, (u32, ())>>();

        let mut iter = vec.iter().rev();

        assert_eq!(iter.next(), Some(&9));
        assert_eq!(iter.next(), Some(&8));
        assert_eq!(iter.next(), Some(&7));
        assert_eq!(iter.next(), Some(&6));
        assert_eq!(iter.next(), Some(&5));
        assert_eq!(iter.next(), Some(&4));
        assert_eq!(iter.next(), Some(&3));
        assert_eq!(iter.next(), Some(&2));
        assert_eq!(iter.next(), Some(&1));
        assert_eq!(iter.next(), Some(&0));
        assert_eq!(iter.next(), None);
    }
    #[test]
    fn convertion_test() {
        use super::*;
        use B;
     
        let mut vec = SelectVec::<u16, (u16, Result<u32, ()>)>::new();
        
        vec.push(10);
        vec.push(20);
        vec.push(30);
        vec.push(40);


        let changed = vec.map::<B, _>(|s| Ok(s as u32) );
        {
            let mut iter = changed.iter();
            assert_eq!(iter.next(), Some(&Ok(10)));
            assert_eq!(iter.next(), Some(&Ok(20)));
            assert_eq!(iter.next(), Some(&Ok(30)));
            assert_eq!(iter.next(), Some(&Ok(40)));
        }

        let clone = changed.clone();
    }

    #[test]
    fn select_type() {
        use super::*;
        use B;

        let mut vec = SelectVec::<char, (char, u8, String)>::new();
        vec.push('a');

        let mut changed = vec.map::<B, _>(|c| c as u8);
        
        changed.push(10);
        assert_eq!(changed.pop(), Some(10));
    }

    #[test]
    fn try_into_vec() {
        use super::*;
        use B;

        let mut vec = SelectVec::<String, (String, u32)>::new();

        vec.push(String::from("10"));
        vec.push(String::from("20"));

        let ints = vec.map::<B, _>(|s| s.parse().unwrap());
        
        assert_eq!(ints.current_type(), type_id::<u32>());

        let mut v = ints.try_to_vec();

        assert_eq!(v.capacity(), 12);
        assert_eq!(v.len(), 2);
        assert_eq!(v.pop(), Some(20));
        assert_eq!(v.pop(), Some(10));
        assert_eq!(v.pop(), None);

    }

    #[test]
    fn try_into_vec_map() {
        use super::*;
        use B;

        let mut vec = SelectVec::<String, (String, u32)>::new();

        vec.push(String::from("10"));
        vec.push(String::from("20"));

        let mut v = vec.try_to_vec_map::<B, _>(|s| s.parse().unwrap());

        assert_eq!(v.capacity(), 12);
        assert_eq!(v.len(), 2);
        assert_eq!(v.pop(), Some(20));
        assert_eq!(v.pop(), Some(10));
        assert_eq!(v.pop(), None);
        
    }
}