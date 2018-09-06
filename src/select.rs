use std::fmt;
use std::mem;
use std::ptr;

use std::marker::PhantomData;
use std::ops::{Deref, DerefMut};

pub(crate) unsafe fn cast_ref<'a, T: 'a, U: 'a>(t_ref: &'a T) -> &'a U {
    &*(t_ref as *const T as *const U)
}

pub(crate) unsafe fn cast_refmut<'a, T: 'a, U: 'a>(t_ref: &'a mut T) -> &'a mut U {
    &mut *(t_ref as *mut T as *mut U)
}

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
        SelectHandle::from(self.cast::<<U as Select<S>>::Output>())
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
    current: PhantomData<T>,
}

impl<T: 'static, U: TypeUnion> SelectHandle<T, U> {
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

    /// Converts `self` into the `T`, and writes the value given into it.
    /// This might be more safe to use than `into`, because the value its written using `ptr::write`.
    #[inline]
    pub fn into_with(mut self, val: T) -> T {
        self.write(val);
        self.into()
    }

    /// Creates a new `SelectHandle` from a Union.
    #[inline]
    pub unsafe fn from_inner(data: U::Union) -> Self {
        Self {
            data,
            current: PhantomData,
        }
    }

    /// Returns the underlying Union.
    #[inline]
    pub fn into_inner(mut self) -> U::Union {
        let data = unsafe { ptr::read(&mut self.data) };
        mem::forget(self);
        data
    }

    /// Copies the current type. It is possible to have a SelectHandle<u64, (u64, String)>, and copy the u64.
    /// This function is not available if T is non-copy
    #[inline]
    pub fn copy_current(&self) -> T
    where
        T: Copy
    {
        *self.deref()
    }

    /// Writes to the underlying value using `ptr::write`.
    #[inline]
    pub fn write(&mut self, item: T) {
        let t = self as *mut Self as *mut T;
        unsafe {
            ptr::write(t, item);
        }
    }

    /// Returns a new handle where the current type is the selected one.
    /// This function drops the *current* held value.
    /// Note that the value contained in the returned handle is undefined,
    /// and should be written to using [`SelectHandle::write`].
    #[inline]
    pub fn change_to<S: Selector>(mut self) -> SelectHandle<<U as Select<S>>::Output, U>
    where
        U: Select<S>
    {
        unsafe {
            ptr::drop_in_place::<T>(self.deref_mut());

            self.into_inner().select::<S>()
        }
    }

    /// Applies the closure on the underlying type, returning a new SelectHandle.
    pub fn map<S: Selector, F: Fn(T) -> <U as Select<S>>::Output>(self, f: F) -> SelectHandle<<U as Select<S>>::Output, U>
    where
        U: Select<S>
    {
        let inner: T = self.into();
        let u: <U as Select<S>>::Output = f(inner);
        SelectHandle::from(u)
    }

    /// Applies the closure on the underlying type.
    /// Returns `Some` if the closure resulted in `Some`, `None` otherwise.
    pub fn filter_map<S, F>(self, f: F) -> Option<SelectHandle<<U as Select<S>>::Output, U>>
    where
        S: Selector,
        U: Select<S>,
        F: Fn(T) -> Option<<U as Select<S>>::Output>
    {
        let inner: T = self.into();

        let maybe = f(inner)?;

        Some(SelectHandle::from(maybe))
    }
}

impl<T1, U1: TypeUnion, T2, U2: TypeUnion> PartialEq<SelectHandle<T1, U1>> for SelectHandle<T2, U2>
where
    // T2: PartialEq<T1>,
    <Self as Deref>::Target: PartialEq<<SelectHandle<T1, U1> as Deref>::Target>
{
    fn eq(&self, other: &SelectHandle<T1, U1>) -> bool {
        (self.deref()).eq(other.deref())
    }
}

impl<T: 'static, U: TypeUnion> From<T> for SelectHandle<T, U> {
    #[inline]
    fn from(t: T) -> Self {
        unsafe { Self::from_unchecked(t) }
    }
}

impl<T, U: TypeUnion> Deref for SelectHandle<T, U> {
    type Target = T;

    #[inline]
    fn deref(&self) -> &Self::Target {
        //unsafe { &*(&self.data as *const <U as TypeUnion>::Union as *const T) }
        //
        unsafe { cast_ref(&self.data) }
    }
}

impl<T, U: TypeUnion> DerefMut for SelectHandle<T, U> {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        //unsafe { &mut *(&mut self.data as *mut <U as TypeUnion>::Union as *mut T) }
        unsafe { cast_refmut(&mut self.data) }
    }
}

impl<T: fmt::Debug, U: TypeUnion> fmt::Debug for SelectHandle<T, U> {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.deref().fmt(f)
    }
}

// @TODO: Fix this.
impl<T: 'static, U: TypeUnion> Clone for SelectHandle<T, U>
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

#[cfg(test)]
mod tests {
    use super::*;
    use index::*;

    #[test]
    fn test_equals() {
        let select_handle_vec = SelectHandle::<Vec<u8>, (Vec<u8>, String)>::from(vec![1, 2, 3]);

        let select_handle_array = SelectHandle::<[u8; 3], ([u8; 3], String)>::from([1, 2, 3]);

        //assert_eq!(select_handle_array, select_handle_vec);
        assert_eq!(select_handle_vec, select_handle_array);
    }

    #[test]
    fn test_into_with() {
        let handle = SelectHandle::<u32, (u32, String)>::from(10u32);
        let handle = handle.change_to::<Type2>();

        let s = handle.into_with(String::new());

        assert_eq!(s, String::new());
    }

    #[test]
    fn test_copy_select_handle() {
        let handle = SelectHandle::<String, (String, u64)>::from(String::from("hi"));

        let mut handle = handle.change_to::<Type2>();
        handle.write(10);
        let copy = handle.copy_current();

        assert_eq!(copy, 10);
    }
}
