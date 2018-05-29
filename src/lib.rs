#![feature(untagged_unions)]
use std::{mem, ptr};
use std::marker::PhantomData;
use std::ops::{Deref, DerefMut, Index, IndexMut};
use std::fmt;

pub union AnyInner<A, B, C> {
    a: A,
    b: B,
    c: C
}

impl <A, B, C> AnyInner<A, B, C> {
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
    #[inline]
    fn is_valid() -> bool {
        use std::any::TypeId;
        let t_id = TypeId::of::<T>();
        let a_id = TypeId::of::<A>();
        let b_id = TypeId::of::<B>();
        let c_id = TypeId::of::<C>();
        t_id == a_id || t_id == b_id || c_id == c_id
    }

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
    pub fn into(mut self) -> T {
        unsafe {
            let t = ptr::read(&mut self as *mut _ as *mut T);
            mem::forget(self);
            t
        }
    }

    #[inline]
    pub fn into_inner(self) -> AnyInner<A, B, C> {
        self.data
    }

    #[inline]
    pub fn from_inner(data: AnyInner<A, B, C>) -> Self {
        assert!(Self::is_valid());
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
    #[inline]
    pub fn is_valid() -> bool {
        AnyItem::<T, A, B, C>::is_valid()
    }
    
    #[inline]
    pub fn new() -> Self {
        assert!(Self::is_valid());
        AnyVec {
            data: Vec::new(),
            marker: PhantomData
        }
    }

    #[inline]
    pub fn with_capacity(capacity: usize) -> Self {
        assert!(Self::is_valid());

        AnyVec {
            data: Vec::with_capacity(capacity),
            marker: PhantomData
        }
    }

    #[inline]
    pub fn capacity(&self) -> usize {
        self.data.capacity()
    }

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

    #[inline]
    pub fn push(&mut self, item: T) {
        self.data.push(AnyItem::from(item).into_inner())
    }
    

    #[inline]
    pub fn iter(&self) -> impl Iterator<Item = &T> {
        self.data.iter().map(|i| unsafe { mem::transmute(i)})
    }

    #[inline]
    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut T> {
        self.data.iter_mut().map(|i| unsafe { mem::transmute(i)})
    }

    #[inline]
    pub fn into_iter(self) -> impl Iterator<Item = T> {
        self.data.into_iter().map(|i| i.select())
    }

    #[inline]
    pub fn pop(&mut self) -> Option<T> {
        self.data.pop().map(|i| AnyItem::<T, A, B, C>::from_inner(i).into())
    }
    
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

    #[inline]
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
        AnyVec { data, marker: PhantomData }
    }
}