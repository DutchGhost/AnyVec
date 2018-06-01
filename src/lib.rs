#![feature(const_fn)]
#![feature(const_type_id)]
#![feature(untagged_unions)]
#![feature(allocator_api)]
#![feature(test)]

extern crate core;
extern crate test;

use std::fmt;
use std::{mem, ptr};
use std::marker::PhantomData;
use std::ops::{Deref, DerefMut, Index, IndexMut};

//pub mod selectvec;
pub mod selectvec;
pub mod macros;

/// This union is used by the [`AnyVec`] struct, to hold the current data-type.
//@TODO: impl drop?
pub union AnyInner<A, B, C> {
    pub a: A,
    pub b: B,
    pub c: C
}

impl <A, B, C> Clone for AnyInner<A, B, C>
where
    A: Clone,
    B: Clone,
    C: Clone,
{
    fn clone(&self) -> Self {
        unsafe {
            match self {
                AnyInner { a } => AnyInner { a: a.clone() }
            }
        }
    }
}
impl <A, B, C> AnyInner<A, B, C> {

    /// Gets out the contained data-type.
    /// # Examples
    /// ```
    /// extern crate anyvec;
    /// use anyvec::AnyInner;
    ///
    /// fn main() {
    ///     let mut inner: AnyInner<String, i32, char> = AnyInner { b: 10i32 };
    ///
    ///     assert_eq!(inner.select::<i32>(), 10);
    /// }
    /// ```
    ///
    /// # Safety
    /// It should be noted that selecting a type that is not current value, results in Undefined Behaviour.
    /// # Example
    /// ```compile_fail
    /// fn main() {
    ///     let mut inner: AnyInner<String, i32, char> = AnyInner { b: 10i32 };
    ///
    ///     assert_eq!(inner.select::<String>(), 10);
    /// }
    /// ```
    #[inline]
    pub fn select<T>(mut self) -> T
    where
        T: 'static,
        A: 'static,
        B: 'static,
        C: 'static,
    {
        assert!(AnyItem::<T, A, B, C>::is_valid());
        unsafe {
            let t = ptr::read(&mut self as *mut _ as *mut T);
            mem::forget(self);
            t
        }
    }

    /// Does the same as [`AnyInner::select()`], but does not type check.
    /// Because no type-check happens, this is considered unsafe, and should be used with care!
    #[inline]
    pub unsafe fn select_unchecked<T>(mut self) -> T {
        let t = ptr::read(&mut self as *mut _ as *mut T);
        mem::forget(self);
        t
    }
}

impl <T, A, B, C> AsRef<T> for AnyInner<A, B, C> {
    #[inline]
    fn as_ref(&self) -> &T {
        unsafe { mem::transmute(self) }
    }
}

impl <T, A, B, C> AsMut<T> for AnyInner<A, B, C> {
    #[inline]
    fn as_mut(&mut self) -> &mut T {
        unsafe { mem::transmute(self) }
    }
}

/// This struct is used by [`AnyVec`], and knows statically what the current data-type is
pub struct AnyItem<T, A, B, C> {
    data: AnyInner<A, B, C>,
    marker: PhantomData<T>
}

impl<T, A, B, C> fmt::Debug for AnyItem<T, A, B, C>
where
    T: fmt::Debug
{
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.deref().fmt(f)
    }
}

impl<T, A, B, C> AnyItem<T, A, B, C>
where
    T: 'static,
    A: 'static,
    B: 'static,
    C: 'static
{
    /// Checks if at least one of the types of A, B and C is equal to the type of T.
    #[inline]
    fn is_valid() -> bool {
        use std::any::TypeId;
        let t_id = TypeId::of::<T>();
        let a_id = TypeId::of::<A>();
        let b_id = TypeId::of::<B>();
        let c_id = TypeId::of::<C>();
        t_id == a_id || t_id == b_id || t_id == c_id
    }

    /// Creates a new instance from `T`.
    #[inline]
    pub fn from(t: T) -> Self {
        assert!(Self::is_valid());
        unsafe {
            let mut s = mem::uninitialized();
            ptr::write(&mut s as *mut _ as *mut T, t);
            s
        }
    }

    #[inline]
    pub unsafe fn from_unchecked(t: T) -> Self {
        let mut s = mem::uninitialized();
        ptr::write(&mut s as *mut _ as *mut T, t);
        s
    }

    /// Converts back to a 'T'.
    #[inline]
    pub fn into(mut self) -> T {
        unsafe {
            let t = ptr::read(&mut self as *mut _ as *mut T);
            mem::forget(self);
            t
        }
    }

    /// Returns the underlying `AnyInner`.
    #[inline]
    pub fn into_inner(self) -> AnyInner<A, B, C> {
        self.data
    }

    /// Creates a new instance from a `AnyInner`.
    #[inline]
    pub fn from_inner(data: AnyInner<A, B, C>) -> Self {
        assert!(Self::is_valid());
        AnyItem {
            data,
            marker: PhantomData
        }
    }

    /// Creates a new instance from a `AnyInner`, but does not type-check.
    #[inline]
    pub unsafe fn from_inner_unchecked(data: AnyInner<A, B, C>) -> Self {
        AnyItem {
            data,
            marker: PhantomData
        }
    }
}

impl <T, A, B, C> AsRef<T> for AnyItem<T, A, B, C>
where
    T: 'static,
    A: 'static,
    B: 'static,
    C: 'static,
{
    #[inline]
    fn as_ref(&self) -> &T {
        assert!(Self::is_valid());
        unsafe { mem::transmute(&self.data) }
    }
}

impl <T, A, B, C> AsMut<T> for AnyItem<T, A, B, C>
where
    T: 'static,
    A: 'static,
    B: 'static,
    C: 'static,
{
    #[inline]
    fn as_mut(&mut self) -> &mut T {
        assert!(Self::is_valid());
        unsafe { mem::transmute(&mut self.data) }
    }
}


impl<T, A, B, C> Deref for AnyItem<T, A, B, C> {
    type Target = T;

    #[inline]
    fn deref(&self) -> &Self::Target {
        unsafe {
            mem::transmute(&self.data)
        }
    }
}

impl <T, A, B, C> DerefMut for AnyItem<T, A, B, C> {

    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe {
            mem::transmute(&mut self.data)
        }
    }
}

/// A vector that can change types, with minimal runtime costs.
/// The current data-type is  defined by `T`,
/// and the data-types it can change to are `A`, `B` and `C`.
///
/// `T` always has to be one of `A`, `B` or `C`.
pub struct AnyVec<T, A, B, C> {
    data: Vec<AnyInner<A, B, C>>,
    marker: PhantomData<T>
}

impl<T, A, B, C> Deref for AnyVec<T, A, B, C> {
    type Target = [AnyItem<T, A, B, C>];

    #[inline]
    fn deref(&self) -> &Self::Target {
        let slice: &[AnyInner<A, B, C>] = self.data.as_ref();
        unsafe {
            // AnyItem is just a wrapper around AnyItem, so this is safe.
            mem::transmute(slice)
        }
    }
}

impl<T, A, B, C> DerefMut for AnyVec<T, A, B, C> {

    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        let slice: &mut [AnyInner<A, B, C>] = self.data.as_mut();
        unsafe {
            mem::transmute(slice)
        }
    }
}

impl <T, A, B, C> Index<usize> for AnyVec<T, A, B, C> {
    type Output = T;

    #[inline]
    fn index(&self, index: usize) -> &Self::Output {
        self.data[index].as_ref()
    }
}

impl <T, A, B, C> IndexMut<usize> for AnyVec<T, A, B, C> {

    #[inline]
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        self.data[index].as_mut()
    }
}

impl<T, A, B, C> AnyVec<T, A, B, C>
where
    T: 'static,
    A: 'static,
    B: 'static,
    C: 'static
{
    /// Checks if any of the types A, B or C is equal the type of T.
    #[inline]
    pub fn is_valid() -> bool {
        AnyItem::<T, A, B, C>::is_valid()
    }

    /// Constructs a new, empty `AnyVec<T, A, B, C>`.
    ///
    /// The underlying vector will not allocate until elements are pushed onto it (see [`AnyVec::push()`]).
    #[inline]
    pub fn new() -> Self {
        assert!(Self::is_valid());
        AnyVec {
            data: Vec::new(),
            marker: PhantomData
        }
    }

    /// Creates a new, empty `AnyVec<T, A, B, C>`, with the specified capacity.
    #[inline]
    pub fn with_capacity(capacity: usize) -> Self {
        assert!(Self::is_valid());

        AnyVec {
            data: Vec::with_capacity(capacity),
            marker: PhantomData
        }
    }

    /// Returns the number of elements the underlying vector can hold without reallocating.
    #[inline]
    pub fn capacity(&self) -> usize {
        self.data.capacity()
    }

    /// Reserves the minimum capacity for exactly `additional` more elements in the given `AnyVec<T, A, B, C>`.
    #[inline]
    pub fn reserve(&mut self, additional: usize) {
        self.data.reserve(additional);
    }

    #[inline]
    pub fn reserve_exact(&mut self, additional: usize) {
        self.data.reserve_exact(additional);
    }

    #[inline]
    pub fn shrink_to_fit(&mut self) {
        self.data.shrink_to_fit();
    }

    #[inline]
    pub fn into_boxed_slice(self) -> Box<[AnyInner<A, B, C>]> {
        self.data.into_boxed_slice()
    }

    #[inline]
    pub fn truncate(&mut self, len: usize) {
        self.data.truncate(len);
    }

    #[inline]
    pub fn as_slice(&self) -> &[AnyItem<T, A, B, C>] {
        self.as_ref()
    }

    #[inline]
    pub fn as_slice_mut(&mut self) -> &mut [AnyItem<T, A, B, C>] {
        self.as_mut()
    }

    #[inline]
    pub unsafe fn set_len(&mut self, len: usize) {
        self.data.set_len(len);
    }

    #[inline]
    pub fn swap_remove(&mut self, index: usize) -> T {
        self.data.swap_remove(index).select()
    }

    #[inline]
    pub fn insert(&mut self, index: usize, item: T) {
        self.data.insert(index, AnyItem::from(item).into_inner());
    }

    #[inline]
    pub fn remove(&mut self, index: usize) -> T {
        self.data.remove(index).select()
    }

    #[inline]
    pub fn retain<F>(&mut self, f: F)
    where
        F: FnMut(&AnyInner<A, B, C>) -> bool
    {
        self.data.retain(f)
    }

    /// Pushes a new element to the vector.
    /// # Examples
    /// ```
    /// use anyvec::AnyVec;
    /// let mut v: AnyVec::<u32, u32, char, ()> = AnyVec::new();
    /// v.push(10);
    /// assert_eq!(v[0], 10);
    /// ```
    /// # Panic
    /// Trying to push an item that is not the current held data-type, will not compile:
    /// ```compile_fail
    /// use anyvec::AnyVec;
    /// let mut v: AnyVec::<u32, u32, char, ()> = AnyVec::new();
    /// v.push('f');
    /// assert_eq!(v[0], 10)
    ///```
    #[inline]
    pub fn push(&mut self, item: T) {
        self.data.push(AnyItem::from(item).into_inner())
    }

    /// Returns an Iterator over the current held data-type.
    #[inline]
    pub fn iter(&self) -> impl Iterator<Item = &T> {
        self.data.iter().map(|i| unsafe { mem::transmute(i)})
    }

    /// Returns an Iterator over the current held data-type that allows mutation.
    #[inline]
    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut T> {
        self.data.iter_mut().map(|i| unsafe { mem::transmute(i)})
    }

    /// Returns an Iterator over the owned items, consuming `self`.
    #[inline]
    pub fn into_iter(self) -> impl Iterator<Item = T> {
        self.data.into_iter().map(|i| unsafe { i.select_unchecked() } )
    }

    /// Pops the first item of the underlying vector. Returns `None` if the vector is empty.
    //@TODO: i.select(), or `AnyItem::<T, A, B, C>::from_inner(i).into()`?
    #[inline]
    pub fn pop(&mut self) -> Option<T> {
        self.data.pop().map(|i| unsafe { i.select_unchecked() } )
    }

    /// Returns a new instance of `AnyVec<U, A, B, C>` by clearing the current vector, leaving the allocated space untouched.
    /// It returns a new `AnyVec<U, A, B, C>`, that can now hold a different data-type.
    /// # Examples
    /// ```
    /// use anyvec::AnyVec;
    /// let mut anyvec = AnyVec::<char, char, u8, String>::new();
    ///
    /// anyvec.push('a');
    ///
    /// let mut changed = anyvec.change_type::<String>();
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
    /// use anyvec::AnyVec;
    /// let mut anyvec = AnyVec::<char, char, u8, String>::new();
    ///
    /// anyvec.push('a');
    ///
    /// let mut changed = anyvec.change_type::<u64>();
    /// changed.push(10);
    /// assert_eq!(changed.pop(), Some(10));
    ///
    /// ```
    //@TODO: change_type() or clear_type() ?
    #[inline]
    pub fn change_type<U>(mut self) -> AnyVec<U, A, B, C> where U: 'static {
        assert!(AnyVec::<U, A, B, C>::is_valid());
        self.data.clear();

        AnyVec {
            data: self.data,
            marker: PhantomData,
        }
    }

    /// This function calls the closure for each element, changing the current datatype in place.
    /// This does not allocate new space.
    /// The new datatype must be a type specified at creation of the AnyVec, otherwise this function will panic.
    /// # Examples
    /// ```
    /// use anyvec::AnyVec;
    ///
    /// let mut vec = AnyVec::<&str, &str, Result<u32, ()>, u32>::new();
    ///
    /// vec.push("10");
    /// vec.push("20");
    /// vec.push("30");
    /// vec.push("40");
    ///
    /// let mut changed = vec.map(|s| s.parse::<u32>().map_err(|_| ()) );
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
    /// let mut final_change = changed.map(|r| r.unwrap());
    ///
    /// let mut iter = final_change.iter();
    /// assert_eq!(iter.next(), Some(&10));
    /// assert_eq!(iter.next(), Some(&20));
    /// assert_eq!(iter.next(), Some(&30));
    /// assert_eq!(iter.next(), Some(&40));
    ///
    /// ```
    #[inline]
    pub fn map<U, F>(self, f: F) -> AnyVec<U, A, B, C>
    where
        U: 'static,
        F: Fn(T) -> U
    {
        assert!(AnyItem::<U, A, B, C>::is_valid());

        let AnyVec { mut data, .. } = self;
        unsafe {
            let ptr = data.as_mut_ptr();
            let len = data.len();
            data.set_len(0);
            for i in 0 .. len as isize {
                let item_ptr = ptr.offset(i);
                let any_t: AnyItem<T, A, B, C> = AnyItem::from_inner_unchecked(ptr::read(item_ptr));
                let t: T = any_t.into();
                let u: U = f(t);
                let any_u: AnyItem<U, A, B, C> = AnyItem::from_unchecked(u);
                ptr::write(item_ptr, any_u.into_inner());
            }
            data.set_len(len);
        }
        AnyVec { data, marker: PhantomData }
    }
}

#[macro_export]
macro_rules! anyvec {
    //@TODO: fix this macro, so one can write: anyvec![10, 10];
    ($elem:expr; $n:expr) => (
        AnyVec {
            data: vec![$crate::AnyItem::from($elem).into_inner(), $n],
            marker: $crate::std::marker::PhantomData,
        }
    );

    ($($elem:expr,)*) => (
        AnyVec {
            data: vec![$($crate::AnyItem::from($elem).into_inner()),*],
            marker: $crate::std::marker::PhantomData,
        }
    );

    ($($elem:expr),*) => ( anyvec!($($elem,)*))
}

#[cfg(test)]
mod tests {
    use AnyVec;

    #[test]
    fn test_map() {
        let mut vec = AnyVec::<&str, &str, Result<u32, ()>, ()>::new();

        vec.push("10");
        vec.push("20");
        vec.push("30");
        vec.push("40");

        let changed = vec.map(|s| s.parse::<u32>().map_err(|_| ()) );

        let mut iter = changed.into_iter();

        assert_eq!(iter.next(), Some(Ok(10)));
        assert_eq!(iter.next(), Some(Ok(20)));
        assert_eq!(iter.next(), Some(Ok(30)));
        assert_eq!(iter.next(), Some(Ok(40)));
    }

    #[test]
    fn test_macro() {
        let mut v: AnyVec<u32, u32, &str, ()> = anyvec![10u32, 20u32, 30u32, 40u32,];
        assert_eq!(v.pop(), Some(40));
        assert_eq!(v.pop(), Some(30));
        assert_eq!(v.pop(), Some(20));
        assert_eq!(v.pop(), Some(10));
        assert_eq!(v.pop(), None);

        //let mut v2: AnyVec<u32, u32, &str, ()> = anyvec![10; 10];
    }
}
