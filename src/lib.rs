#![feature(untagged_unions)]
use std::{mem, ptr};
use std::marker::PhantomData;
use std::ops::Deref;
use std::fmt;

pub union AnyInner<A, B, C> {
    a: A,
    b: B,
    c: C
}

pub struct AnyItem<T, A, B, C> {
    data: AnyInner<A, B, C>,
    _m: PhantomData<T>
}

impl<T, A, B, C> fmt::Debug for AnyItem<T, A, B, C>
where
    T: fmt::Debug
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.deref().fmt(f)
    }
}

impl<T, A, B, C> AnyItem<T, A, B, C> where T: 'static, A: 'static, B: 'static, C: 'static {
    fn is_valid() -> bool {
        use std::any::TypeId;
        let t_id = TypeId::of::<T>();
        let a_id = TypeId::of::<A>();
        let b_id = TypeId::of::<B>();
        let c_id = TypeId::of::<C>();
        t_id == a_id || t_id == b_id || c_id == c_id
    }

    pub fn from(t: T) -> Self {
        assert!(Self::is_valid());
        unsafe {
            let mut s = mem::uninitialized();
            ptr::write(&mut s as *mut _ as *mut T, t);
            s
        }
    }

    pub fn into(mut self) -> T {
        unsafe {
            let t = ptr::read(&mut self as *mut _ as *mut T);
            mem::forget(self);
            t
        }
    }

    pub fn into_inner(self) -> AnyInner<A, B, C> {
        self.data
    }

    pub fn from_inner(data: AnyInner<A, B, C>) -> Self {
        assert!(Self::is_valid());
        AnyItem {
            data,
            _m: PhantomData
        }
    }
}

impl<T, A, B, C> Deref for AnyItem<T, A, B, C> {
    type Target = T;
    fn deref(&self) -> &T {
        unsafe { 
            mem::transmute(&self.data)
        }
    }
}

pub struct AnyVec<T, A, B, C> {
    data: Vec<AnyInner<A, B, C>>,
    _m: PhantomData<T>
}

impl<T, A, B, C> Deref for AnyVec<T, A, B, C> {
    type Target = [AnyItem<T, A, B, C>];
    fn deref(&self) -> &Self::Target {
        let slice: &[AnyInner<A, B, C>] = &self.data[..];
        unsafe {
            // AnyItem is just a wrapper around AnyItem, so this is safe.
            mem::transmute(slice)
        }
    }
}

impl<T, A, B, C> AnyVec<T, A, B, C> where T: 'static, A: 'static, B: 'static, C: 'static {
    pub fn is_valid() -> bool {
        AnyItem::<T, A, B, C>::is_valid()
    }
    pub fn new() -> Self {
        assert!(Self::is_valid());
        AnyVec {
            data: Vec::new(),
            _m: PhantomData
        }
    }

    pub fn push(&mut self, item: T) {
        self.data.push(AnyItem::from(item).into_inner())
    }
    
    pub fn iter(&self) -> impl Iterator<Item = &AnyItem<T, A, B, C>> {
        self.data.iter().map(|i| unsafe { mem::transmute(i)})
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut AnyItem<T, A, B, C>> {
        self.data.iter_mut().map(|i| unsafe { mem::transmute(i)})
    }

    pub fn pop(&mut self) -> Option<AnyItem<T, A, B, C>> {
        self.data.pop().map(|i| AnyItem::from_inner(i))
    }
    
    pub fn reset<U>(mut self) -> AnyVec<U, A, B, C> where U: 'static {
        assert!(AnyVec::<U, A, B, C>::is_valid());
        self.data.clear();
        
        AnyVec {
            data: self.data,
            _m: PhantomData,
        }
    }
}