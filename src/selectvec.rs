use std::ptr;
use std::mem;
use std::fmt;
use std::any::TypeId;
use std::marker::PhantomData;
use std::convert::{AsRef, AsMut};

pub union SelectUnion<A, B, C> {
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

/// This trait is used to check at runtime whether any type `T` equals one of a sequence of other types.
/// Part of this check can happen during compilation, since we know the types of `T` and the sequence at compile time,
/// the only part at runtime is comparison
pub trait TypeEq {
    type Type: Sized;

    /// Returns `true` if `T` is one of a sequence of other types.
    fn contains<T: 'static>() -> bool;
}

impl <A, B, C> TypeEq for (A, B, C)
where
    A: 'static,
    B: 'static,
    C: 'static,
{
    type Type = SelectUnion<A, B, C>;

    fn contains<T: 'static>() -> bool {
        contains_type!(T, [A, B, C])
    }
}

pub struct A;
pub struct B;
pub struct C;

pub trait Selector {}

impl Selector for A {}
impl Selector for B {}
impl Selector for C {}

/// This trait can be used to 'select' a current type.
pub trait Select<S> {

    /// The current selected type.
    type Output;
}

impl <AA, BB, CC> Select<A> for (AA, BB, CC) {
    type Output = AA;
}

impl <AA, BB, CC> Select<B> for (AA, BB, CC) {
    type Output = BB;
}

impl <AA, BB, CC> Select<C> for (AA, BB, CC) {
    type Output = CC;
}

impl <A, B, C> SelectUnion<A, B, C>
{
    #[inline]
    pub fn select<S: Selector>(mut self) -> <(A, B, C) as Select<S>>::Output
    where
        (A, B, C): Select<S>
    {
        unsafe {
            let t = ptr::read(&mut self as *mut _ as *mut <(A, B, C) as Select<S>>::Output);
            mem::forget(self);
            t
        }
    }
}

impl <T, A, B, C> AsRef<T> for SelectUnion<A, B, C> {
    #[inline]
    fn as_ref(&self) -> &T {
        unsafe { mem::transmute(self) }
    }
}

impl <T, A, B, C> AsMut<T> for SelectUnion<A, B, C> {
    #[inline]
    fn as_mut(&mut self) -> &mut T {
        unsafe { mem::transmute(self) }
    }
}

/// Struct to safely to from [`SelectUnion`] to `T`.
/// 
/// # Examples
/// ```
/// use anyvec::selectvec::{SelectItem, Selector, A, B, C};
/// 
/// let mut item: SelectItem<u32, u32, String, ()> = SelectItem::from::<A>(10);
/// ```
pub struct SelectItem<T, A, B, C>
{
    data: SelectUnion<A, B, C>,
    marker: PhantomData<T>,
}

impl <T, A, B, C> SelectItem<T, A, B, C> {
    #[inline]
    pub fn from<S: Selector>(t: T) -> Self
    where
        (A, B, C): Select<S, Output = T>
    {
        unsafe {
            let mut s = mem::uninitialized();
            ptr::write(&mut s as *mut _ as *mut T, t);
            s
        }
    }

    #[inline]
    pub fn into(mut self) -> T {
        unsafe {
            let t = ptr::read(&mut self as *mut _ as *mut T);
            mem::forget(self);
            t
        }
    }

    #[inline]
    pub fn into_inner(self) -> SelectUnion<A, B, C> {
        self.data
    }

    #[inline]
    pub fn from_inner(data: SelectUnion<A, B, C>) -> SelectItem<T, A, B, C> {
        SelectItem {
            data,
            marker: PhantomData,
        }
    }
}

impl <T, A, B, C> AsRef<T> for SelectItem<T, A, B, C> {
    #[inline]
    fn as_ref(&self) -> &T {
        self.data.as_ref()
    }
}

impl <T, A, B, C> AsMut<T> for SelectItem<T, A, B, C> {
    #[inline]
    fn as_mut(&mut self) -> &mut T {
        self.data.as_mut()
    }
}

impl <T, A, B, C> fmt::Debug for SelectItem<T, A, B, C>
where
    T: fmt::Debug
{
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.as_ref().fmt(f)
    }
}

/// A Vector that can hold multiple data-types, and switch to those data-types, without losing its allocated space.
pub struct SelectVec<T, A, B, C> {
    data: Vec<SelectUnion<A, B, C>>,
    marker: PhantomData<T>
}

impl <T, A, B, C> SelectVec<T, A, B, C> {
    
    /// Creates a new empty `SelectVec<T, A, B, C>`.
    #[inline]
    pub fn new() -> Self {
        SelectVec {
            data: Vec::new(),
            marker: PhantomData,
        }
    }

    /// Pushes a new element to the vector.
    /// # Examples
    /// ```
    /// use anyvec::selectvec::{SelectVec, A, B, C};
    /// let mut vec = SelectVec::<char, char, u8, String>::new();
    ///
    /// vec.push::<A>('a');
    /// vec.push::<A>('b');
    /// 
    /// assert_eq!(vec.pop::<A>(), Some('b'));
    /// assert_eq!(vec.pop::<A>(), Some('a'));
    /// assert_eq!(vec.pop::<A>(), None);
    ///```
    #[inline]
    pub fn push<S: Selector>(&mut self, item: T)
    where
        (A, B, C): Select<S, Output = T>
    {
        self.data.push(SelectItem::from::<S>(item).into_inner());
    }
    
    #[inline]
    pub fn pop<S: Selector>(&mut self) -> Option<T>
    where
        (A, B, C): Select<S, Output = T>
    {
        self.data.pop().map(|i| i.select::<S>())
    }

    #[inline]
    pub fn iter(&self) -> impl Iterator<Item = &T> {
        self.data.iter().map(|item| unsafe { mem::transmute(item) })
    }

    #[inline]
    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut T> {
        self.data.iter_mut().map(|item| unsafe { mem::transmute(item) })
    }

    #[inline]
    pub fn into_iter<S: Selector>(self) -> impl Iterator<Item = T>
    where
        (A, B, C): Select<S, Output = T>
    {
        self.data.into_iter().map(|item| item.select::<S>())
    }
    /// Returns a new instance of `AnyVec<U, A, B, C>` by clearing the current vector, leaving the allocated space untouched.
    /// It returns a new `AnyVec<U, A, B, C>`, that can now hold a different data-type.
    /// # Examples
    /// ```
    /// use anyvec::selectvec::{SelectVec, A, B, C};
    /// let mut vec = SelectVec::<char, char, u8, String>::new();
    ///
    /// vec.push::<A>('a');
    ///
    /// let mut changed = vec.change_type::<C, String>();
    ///
    /// changed.push::<C>(String::from("foo"));
    ///
    /// assert_eq!(changed.pop::<C>(), Some(String::from("foo")));
    /// ```
    ///
    /// # Panic
    /// Trying to change to a datatype that is not specified at creation, is not allowed, and will result in a panic!():
    ///
    /// ```comptime_fail
    /// use anyvec::selectvec{SelectVec, A, B, C};
    /// let mut vec = SelectVec::<char, char, u8, String>::new();
    ///
    /// vec.push::<A>('a');
    ///
    /// let mut changed = vec.change_type::<B, _>();
    /// changed.push::<B>(10);
    /// assert_eq!(changed.pop::<B>(), Some(10));
    ///
    /// ```
    #[inline]
    pub fn change_type<S, U>(mut self) -> SelectVec<U, A, B, C>
    where
        (A, B, C): Select<S, Output = U>
    {
        self.data.clear();

        SelectVec {
            data: self.data,
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
    /// let mut vec = SelectVec::<&str, &str, Result<u32, ()>, u32>::new();
    ///
    /// vec.push::<A>("10");
    /// vec.push::<A>("20");
    /// vec.push::<A>("30");
    /// vec.push::<A>("40");
    ///
    /// let mut result_vec = vec.map::<_, B, _>(|s| s.parse::<u32>().map_err(|_| ()) );
    ///
    /// {
    ///     let mut iter = result_vec.iter();
    ///
    ///     assert_eq!(iter.next(), Some(&Ok(10)));
    ///     assert_eq!(iter.next(), Some(&Ok(20)));
    ///     assert_eq!(iter.next(), Some(&Ok(30)));
    ///     assert_eq!(iter.next(), Some(&Ok(40)));
    /// }
    ///
    /// let mut int_vec = result_vec.map::<_, C, _>(|r| r.unwrap());
    /// 
    /// let mut iter = int_vec.into_iter::<C>();
    /// 
    /// assert_eq!(iter.next(), Some(10));
    /// assert_eq!(iter.next(), Some(20));
    /// assert_eq!(iter.next(), Some(30));
    /// assert_eq!(iter.next(), Some(40));
    /// ```
    #[inline]
    pub fn map<U, S: Selector, F: Fn(T) -> U>(self, f: F) -> SelectVec<U, A, B, C>
    where
        (A, B, C): Select<S, Output = U>
    {
        let SelectVec { mut data, ..} = self;

        unsafe {
            let ptr = data.as_mut_ptr();
            let len = data.len();
            data.set_len(0);

            for i in 0..len as isize {
                let item_ptr = ptr.offset(i);
                let any_t: SelectItem<T, A, B, C> = SelectItem::from_inner(ptr::read(item_ptr));
                let t: T = any_t.into();
                let u: U = f(t);
                let any_u: SelectItem<U, A, B, C> = SelectItem::from::<S>(u);
                ptr::write(item_ptr, any_u.into_inner());
            }

            data.set_len(len);
        }

        SelectVec {data, marker: PhantomData}
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn wtf() {
        use super::*;
        let mut vec = SelectVec::<&str, &str, Result<u32, ()>, u32>::new();
        
        vec.push::<A>("10");
        vec.push::<A>("20");
        vec.push::<A>("30");
        vec.push::<A>("40");
        
        let mut changed = vec.map::<_, C, _>(|s| s.parse::<u32>().unwrap() );
        {
            let mut iter = changed.iter();
            assert_eq!(iter.next(), Some(&10));
            assert_eq!(iter.next(), Some(&20));
            assert_eq!(iter.next(), Some(&30));
            assert_eq!(iter.next(), Some(&40));
        }
    }
}