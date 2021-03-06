use std::fmt;
use std::mem;
use std::ptr;

use std::marker::PhantomData;
use std::ops::{Deref, DerefMut};

/// Helper trait to index into a tuple of Generics.
pub trait Selector {}

/// Helper trait to 'select' a generic type out of a tuple of Generics.
pub trait Select<S: Selector> {
    /// The selected generic.
    type Output: 'static;
}

/// This trait offers functions to cast any type, into any other type, constraintly.
pub unsafe trait TypeSelect<U: TypeUnion>: Sized {
    /// Casts `self` to `T`.
    ///
    /// # Panic
    ///
    /// When `self` can not be safely cast to `T`, this function will panic.
    #[inline]
    unsafe fn cast<T: 'static>(self) -> T {
        debug_assert!(U::contains::<T>());
        let mut s = mem::uninitialized();
        ptr::write(&mut s as *mut _ as *mut Self, self);
        s
    }

    #[inline]
    unsafe fn select<S>(self) -> SelectHandle<<U as Select<S>>::Output, U>
    where
        S: Selector,
        U: Select<S>,
    {
        self.cast()
    }
}

pub trait TypeUnion: Sized + 'static {
    type Union: TypeSelect<Self>;

    /// This function should only be used with tuples.
    ///
    /// Returns `true` if T is one of the types of the tuple, false otherwise.
    fn contains<T: 'static>() -> bool;
}

/// A wrapper around Unions, that keeps track of the current type using PhantomData.
pub struct SelectHandle<T, U: TypeUnion> {
    /// The Union itself.
    data: U::Union,

    /// A marker field, indicating the current type of the union.
    marker: PhantomData<T>,
}

impl<T, U: TypeUnion> SelectHandle<T, U> {
    /// Creates a new Union, and writes the given value to it.
    #[inline]
    pub unsafe fn from_unchecked(t: T) -> Self {
        let mut s = mem::uninitialized();
        ptr::write(&mut s as *mut _ as *mut T, t);
        s
    }

    /// Converts `self` into `T`.
    #[inline]
    pub fn into(mut self) -> T {
        unsafe {
            let t = ptr::read(&mut self as *mut _ as *mut T);
            mem::forget(self);
            t
        }
    }

    /// Creates a new `SelectHandle` from a Union.
    #[inline]
    pub unsafe fn from_inner(data: U::Union) -> Self {
        Self {
            data,
            marker: PhantomData,
        }
    }

    /// Returns the underlying Union.
    #[inline]
    pub fn into_inner(mut self) -> U::Union {
        let data = unsafe { ptr::read(&mut self.data) };
        mem::forget(self);
        data
    }
}

impl<T, U: TypeUnion> From<T> for SelectHandle<T, U> {
    #[inline]
    fn from(t: T) -> Self {
        unsafe { Self::from_unchecked(t) }
    }
}

impl<T, U: TypeUnion> Deref for SelectHandle<T, U> {
    type Target = T;

    #[inline]
    fn deref(&self) -> &Self::Target {
        unsafe { &*(&self.data as *const <U as TypeUnion>::Union as *const T) }
    }
}

impl<T, U: TypeUnion> DerefMut for SelectHandle<T, U> {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { &mut *(&mut self.data as *mut <U as TypeUnion>::Union as *mut T) }
    }
}

impl<T: fmt::Debug, U: TypeUnion> fmt::Debug for SelectHandle<T, U> {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.deref().fmt(f)
    }
}

// @TODO: Fix this.
impl<T, U: TypeUnion> Clone for SelectHandle<T, U>
where
    T: Clone,
{
    #[inline]
    fn clone(&self) -> Self {
        let clone_of_t = self.deref().clone();

        Self::from(clone_of_t)
    }
}

impl<T, U: TypeUnion> Drop for SelectHandle<T, U> {
    fn drop(&mut self) {
        // `T` is the current held type.
        unsafe {
            ptr::drop_in_place::<T>(self.deref_mut());
        }
    }
}
