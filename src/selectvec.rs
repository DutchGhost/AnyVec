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

pub struct Phantom1;
pub struct Phantom2;
pub struct Phantom3;

pub trait PhantomTrait {}

impl PhantomTrait for Phantom1 {}
impl PhantomTrait for Phantom2 {}
impl PhantomTrait for Phantom3 {}

/// This trait can be used to 'select' a current type.
pub trait Select<S> {

    /// The current selected type.
    type Output;
}

impl <A, B, C> Select<Phantom1> for (A, B, C) {
    type Output = A;
}

impl <A, B, C> Select<Phantom2> for (A, B, C) {
    type Output = B;
}

impl <A, B, C> Select<Phantom3> for (A, B, C) {
    type Output = C;
}

impl <A, B, C> SelectUnion<A, B, C>
where
    A: 'static,
    B: 'static,
    C: 'static,
{
    /// Converts self into `T`.
    /// `T` must be one of `A`, `B` and `C`.
    /// 
    /// # Panic
    /// Panics if `T` is not one of `A`, `B` and `C`.
    #[inline]
    pub fn select<T: 'static>(self) -> T {
        assert!(<(A, B, C)>::contains::<T>());

        unsafe {
            self.select_unchecked::<T>()
        }
    }

    /// Converts self into `T`.
    /// 
    /// # Safety
    /// This function does not type-check if T is one of `A`, `B` and `C`.
    /// If `T` is not one of those, this caused Undefined Behaviour.
    #[inline]
    pub unsafe fn select_unchecked<T>(mut self) -> T {
        let t = ptr::read(&mut self as *mut _ as *mut T);
        mem::forget(self);
        t
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
/// use anyvec::selectvec::{SelectItem, SelectItemExt};
/// 
/// let mut item: SelectItem<u32, u32, String, ()> = SelectItemExt::from(10);
/// ```
pub struct SelectItem<T, A, B, C>
{
    data: SelectUnion<A, B, C>,
    marker: PhantomData<T>,
}

pub trait SelectItemExt<T, S: PhantomTrait> {
    fn from(t: T) -> Self;

    fn into(self) -> T;
}

macro_rules! impl_select_item {
    ($phantom:ty) => (
        impl <T, A, B, C> SelectItemExt<T, $phantom> for SelectItem<T, A, B, C>
        where
            (A, B, C): TypeEq<Type = SelectUnion<A, B, C>> + Select<$phantom, Output = T>
        {
            fn from(t: T) -> Self {
                unsafe {
                    let mut s = mem::uninitialized();
                    ptr::write(&mut s as *mut _ as *mut T, t);
                    s
                }
            }

            fn into(mut self) -> T {
                unsafe {
                    let t = ptr::read(&mut self as *mut _ as *mut T);
                    mem::forget(self);
                    t
                }
            }
        }
    )
}

impl_select_item!(Phantom1);
impl_select_item!(Phantom2);
impl_select_item!(Phantom3);
