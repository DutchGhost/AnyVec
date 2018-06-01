use std::ptr;
use std::mem;
use std::fmt;
use std::iter;
use std::any::TypeId;
use std::marker::PhantomData;
use std::convert::{AsRef, AsMut};

pub union Union3<A, B, C> {
    a: A,
    b: B,
    c: C,
}

macro_rules! contains_type {
    ($T:ty, [$($O:ty),*]) => (
        false $(|| type_id::<$T>() == type_id::<$O>())*
    )
}

/// Returns the TypeId of `T`.
pub const fn type_id<T: 'static>() -> TypeId {
    TypeId::of::<T>()
}

pub unsafe trait TypeSelect<U: TypeUnion> : Sized {
    
    /// Casts `self` to `T`.
    /// This should only be used in context with [`Union3`],
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
unsafe impl<A, B, C> TypeSelect<(A, B, C)> for Union3<A, B, C> 
    where A: 'static, B: 'static, C: 'static
{}

/// This trait is used to check at runtime whether any type `T` equals one of a sequence of other types.
/// Part of this check can happen during compilation, since we know the types of `T` and the sequence at compile time,
/// the only part at runtime is the comparison.
pub trait TypeUnion: Sized + 'static {
    type Union: TypeSelect<Self>;

    /// Returns `true` if `T` is one of a sequence of other types.
    fn contains<T: 'static>() -> bool;

}

impl <A, B, C> TypeUnion for (A, B, C)
where
    A: 'static,
    B: 'static,
    C: 'static,
{
    type Union = Union3<A, B, C>;

    #[inline]
    fn contains<T: 'static>() -> bool {
        contains_type!(T, [A, B, C])
    }
}

#[derive(Debug, Ord, PartialOrd, Hash, Eq, PartialEq, Default)]
pub struct A;

#[derive(Debug, Ord, PartialOrd, Hash, Eq, PartialEq, Default)]
pub struct B;

#[derive(Debug, Ord, PartialOrd, Hash, Eq, PartialEq, Default)]
pub struct C;

/// Helper trait to index into a tuple of Generics.
pub trait Selector {}

impl Selector for A {}
impl Selector for B {}
impl Selector for C {}

/// Helper trait to 'select' a generic type out of a tuple of Generics.
pub trait Select<S: Selector> {

    /// The current selected type.
    type Output: 'static;
}

impl <AA, BB, CC> Select<A> for (AA, BB, CC) where AA: 'static {
    type Output = AA;
}

impl <AA, BB, CC> Select<B> for (AA, BB, CC) where BB: 'static  {
    type Output = BB;
}

impl <AA, BB, CC> Select<C> for (AA, BB, CC) where CC: 'static  {
    type Output = CC;
}

/// Struct to safely to from [`Union3`] to `T`.
/// 
/// # Examples
/// ```
/// use anyvec::selectvec::{SelectItem, Selector, A, B, C};
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

/// A Vector that can hold multiple data-types, and switch to those data-types, without losing its allocated space.
pub struct SelectVec<T, D>
where
    T: 'static,
    D: TypeUnion
{
    data: Vec<D::Union>,
    marker: PhantomData<T>
}

impl<T, D> SelectVec<T, D> where D: TypeUnion, T: 'static {
    
    /// Creates a new empty `SelectVec<T, A, B, C>`.
    #[inline]
    pub fn new() -> Self {
        SelectVec {
            data: Vec::new(),
            marker: PhantomData,
        }
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

    /// Returns a by-reference Iterator over the items contained in the Vector.
    #[inline]
    pub fn iter(&self) -> impl Iterator<Item = &T> {
        self.data.iter().map(|item| unsafe { mem::transmute(item) })
    }

    /// Returns a by-mutable-reference Iterator over the items contained in the Vector.
    /// This allows for mutation.
    #[inline]
    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut T> {
        self.data.iter_mut().map(|item| unsafe { mem::transmute(item) })
    }

    /// Returns a by-value Iterator over the items contained in the Vector.
    #[inline]
    pub fn into_iter(self) -> impl Iterator<Item = T>
    {
        self.into_data().into_iter().map(|i| unsafe {
            let item = SelectItem::<T, D>::from_inner(i);
            item.into()
        })
    }

    /// Returns a draining Iterator, consuming the range of items specified.
    #[inline]
    pub fn drain<'a, R>(&'a mut self, r: R) -> impl Iterator<Item = T> + 'a
    where
        R: ::std::ops::RangeBounds<usize>
    {
        self.data.drain(r).map(move |i| unsafe {
            let item = SelectItem::<T, D>::from_inner(i);
            item.into()
        })
    }

    /// Returns a new instance of `AnyVec<U, A, B, C>` by clearing the current vector, leaving the allocated space untouched.
    /// It returns a new `AnyVec<U, A, B, C>`, that can now hold a different data-type.
    /// # Examples
    /// ```
    /// use anyvec::selectvec::{SelectVec, A, B, C};
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
    /// use anyvec::selectvec{SelectVec, A, B, C};
    /// let mut vec = SelectVec::<char, (char, u8, String)>::new();
    ///
    /// vec.push('a');
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

    /// This function calls the closure for each element, changing the current datatype in place.
    /// This does not allocate new space.
    /// The new datatype must be a type specified at creation of the AnyVec, otherwise this function will panic.
    /// # Examples
    /// ```
    /// use anyvec::selectvec::{SelectVec, A, B, C};
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
    ///
    /// ```
    /// 
    /// # Safety
    /// If the closure panics, the internal vector is leaked.
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

    //@TODO: FIXME
    #[inline]
    pub fn try_to_vec<S: Selector>(self) -> Vec<T>
    where
        D: Select<S>
    {
        let mut data = self.into_data();

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

            data.set_len(len);
            Vec::from_raw_parts(base_write_ptr, len, len)
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

#[cfg(test)]
mod tests {
    #[test]
    fn convertion_test() {
        use super::*;
     
        let mut vec = SelectVec::<u16, (u16, Result<u32, ()>, u32)>::new();
        
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
    }

    #[test]
    fn select_wrong_type() {
        use super::*;
        
        let mut vec = SelectVec::<char, (char, u8, String)>::new();
        vec.push('a');
        let mut changed = vec.map::<B, _>(|c| c as u8);
        changed.push(10);
        assert_eq!(changed.pop(), Some(10));
    }
}