use std::mem;
use std::ptr;
use std::ops::{Deref, DerefMut};
use std::marker::PhantomData;

use select_core::select::{Select, SelectHandle, TypeUnion, Selector};

/// A heap allocated, tracked union.
#[derive(Ord, PartialOrd, Eq, PartialEq, Default, Hash)]
pub struct UnionBox<T: 'static, U: TypeUnion> {
    data: Box<U::Union>,
    marker: PhantomData<T>,
}

impl <T: 'static, U: TypeUnion> UnionBox<T, U> {
    #[inline]
    pub fn new(x: T) -> Self {
        Self {
            data: Box::new(SelectHandle::<T, U>::from(x).into_inner()),
            marker: PhantomData,
        }
    }

    #[inline]
    pub fn into_data(self) -> Box<U::Union> {
        let data = unsafe { ptr::read(&self.data) };
        mem::forget(self);
        data
    }

    #[inline]
    pub fn map<S, F>(self, f: F) -> UnionBox<<U as Select<S>>::Output, U>
    where
        S: Selector,
        U: Select<S>,
        F: Fn(T) -> <U as Select<S>>::Output
    {
        let data = self.into_data();

        let ptr = Box::into_raw(data);

        unsafe {
            let union_t: SelectHandle<T, U> = SelectHandle::from_inner(ptr::read(ptr));
            let union_u: SelectHandle<<U as Select<S>>::Output, U> = union_t.map::<S, _>(&f);

            ptr::write(ptr, union_u.into_inner());

            UnionBox {
                data: Box::from_raw(ptr),
                marker: PhantomData,
            }
        }
    }
}

impl <T, U: TypeUnion> Deref for UnionBox<T, U> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        let union_t = self.data.deref();

        unsafe { &*(union_t as *const _ as *const T) }
    }
}

impl <T, U: TypeUnion> DerefMut for UnionBox<T, U> {

    fn deref_mut(&mut self) -> &mut Self::Target {
        let union_t = self.data.deref_mut();

        unsafe { &mut *(union_t as *mut _ as *mut T) }
    }
}


impl <T, U: TypeUnion> Clone for UnionBox<T, U>
where
    T: Clone,
{
    fn clone(&self) -> Self {
        let t: &T = self.deref();

        let clone_t = t.clone();

        Self::new(clone_t)
    }
}

impl <T, U: TypeUnion> Drop for UnionBox<T, U>
{
    fn drop(&mut self) {
        unsafe {
            ptr::drop_in_place::<T>(self.deref_mut())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use select_core::index::Type2;

    #[test]
    fn test_unionbox_map() {
        let b = UnionBox::<&str, (&str, u64)>::new("200");

        let b = b.map::<Type2, _>(|item| item.parse().unwrap());

        assert_eq!(*b, 200);
    }
}
