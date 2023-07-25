use crate::libs::kernel_linked_list::{LinkedList, Node};
use core::cell::UnsafeCell;
use core::ops::{Deref, DerefMut};
use core::sync::atomic::{AtomicBool, Ordering};

struct Mutex<T> {
    inner: InnerMutex<T>,
}

pub struct InnerMutex<T: Sized> {
    pub(crate) lock: AtomicBool,
    data: UnsafeCell<T>,
    waite_list: LinkedList<Node<()>>,
}

pub struct MutexGuard<'a, T: 'a + ?Sized> {
    inner: InnerMutexGuard<'a, T>,
}

pub struct InnerMutexGuard<'a, T: ?Sized + 'a> {
    lock: &'a AtomicBool,
    data: *mut T,
}

impl<T> Mutex<T> {
    #[inline(always)]
    pub const fn new(value: T) -> Self {
        Self {
            inner: InnerMutex::new(value),
        }
    }

    pub fn is_locked(&self) -> bool {
        self.inner.is_locked()
    }

    pub fn lock(&self) -> MutexGuard<T> {
        MutexGuard {
            inner: self.inner.lock(),
        }
    }
}

impl<T> InnerMutex<T> {
    #[inline(always)]
    pub const fn new(data: T) -> Self {
        InnerMutex {
            lock: AtomicBool::new(false),
            data: UnsafeCell::new(data),
            waite_list: LinkedList::new(),
        }
    }

    // 关键方法,上锁
    #[inline(always)]
    pub fn lock(&self) -> InnerMutexGuard<T> {
        // todo
        InnerMutexGuard {
            lock: &self.lock,
            data: unsafe { &mut *self.data.get() },
        }
    }

    pub fn is_locked(&self) -> bool {
        self.lock.load(Ordering::Relaxed)
    }
}

// 关键方法,离开作用域自动解锁
impl<'a, T: ?Sized> Drop for InnerMutexGuard<'a, T> {
    fn drop(&mut self) {
        todo!()
    }
}

impl<'a, T: ?Sized> Deref for MutexGuard<'a, T> {
    type Target = T;
    fn deref(&self) -> &T {
        &self.inner
    }
}

impl<'a, T: ?Sized> DerefMut for MutexGuard<'a, T> {
    fn deref_mut(&mut self) -> &mut T {
        &mut self.inner
    }
}

impl<'a, T: ?Sized> Deref for InnerMutexGuard<'a, T> {
    type Target = T;
    fn deref(&self) -> &T {
        unsafe { &*self.data }
    }
}

impl<'a, T: ?Sized> DerefMut for InnerMutexGuard<'a, T> {
    fn deref_mut(&mut self) -> &mut T {
        unsafe { &mut *self.data }
    }
}
