#![feature(untagged_unions)]
use std::{mem, ptr};
use std::marker::PhantomData;
use std::ops::{Deref, DerefMut};
use std::fmt;

pub union AnyInner<A, B, C> {
    a: A,
    b: B,
    c: C
}

impl <A, B, C> AnyInner<A, B, C> {
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

impl<T, A, B, C> AnyItem<T, A, B, C>
where
    T: 'static,
    A: 'static,
    B: 'static,
    C: 'static
{
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
    fn deref(&self) -> &Self::Target {
        unsafe { 
            mem::transmute(&self.data)
        }
    }
}

impl <T, A, B, C> DerefMut for AnyItem<T, A, B, C> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe {
            mem::transmute(&mut self.data)
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

impl<T, A, B, C> AnyVec<T, A, B, C>
where
    T: 'static,
    A: 'static,
    B: 'static,
    C: 'static
{
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
    
    pub fn iter(&self) -> impl Iterator<Item = &T> {
        self.data.iter().map(|i| unsafe { mem::transmute(i)})
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut T> {
        self.data.iter_mut().map(|i| unsafe { mem::transmute(i)})
    }

    pub fn into_iter(self) -> impl Iterator<Item = T> {
        self.data.into_iter().map(|i| i.select())
    }

    pub fn pop(&mut self) -> Option<T> {
        self.data.pop().map(|i| AnyItem::<T, A, B, C>::from_inner(i).into())
    }
    
    pub fn reset<U>(mut self) -> AnyVec<U, A, B, C> where U: 'static {
        assert!(AnyVec::<U, A, B, C>::is_valid());
        self.data.clear();
        
        AnyVec {
            data: self.data,
            _m: PhantomData,
        }
    }

    pub fn map<U, F>(self, f: F) -> AnyVec<U, A, B, C> where U: 'static, F: Fn(T) -> U
    {
        let AnyVec { mut data, .. } = self;
        unsafe {
            let ptr = data.as_mut_ptr();
            let len = data.len();
            data.set_len(0);
            for i in 0 .. len as isize {
                let item_ptr = ptr.offset(i);
                let any_t: AnyItem<T, A, B, C> = AnyItem::from_inner(ptr::read(item_ptr));
                let t: T = any_t.into();
                let u: U = f(t);
                let any_u: AnyItem<U, A, B, C> = AnyItem::from(u);
                ptr::write(item_ptr, any_u.into_inner());
            }
            data.set_len(len);
        }
        AnyVec { data, _m: PhantomData }
    }
}