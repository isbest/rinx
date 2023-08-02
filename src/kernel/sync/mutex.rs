use crate::kernel::interrupts::without_interrupt;
use crate::kernel::system_call::sys_call::sys_yield;
use crate::kernel::tasks::task::{Task, TaskState};
use crate::libs::kernel_linked_list::LinkedList;
use core::cell::UnsafeCell;
use core::ops::{Deref, DerefMut};

pub struct Mutex<T> {
    inner: InnerMutex<T>,
}

pub struct InnerMutex<T: ?Sized> {
    pub(crate) lock: UnsafeCell<bool>,
    waite_list: UnsafeCell<LinkedList<()>>,
    data: UnsafeCell<T>,
}

pub struct MutexGuard<'a, T: 'a + ?Sized> {
    inner: InnerMutexGuard<'a, T>,
}

pub struct InnerMutexGuard<'a, T: ?Sized + 'a> {
    lock: &'a UnsafeCell<bool>,
    data: *mut T,
    waite_list: &'a UnsafeCell<LinkedList<()>>,
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
            lock: UnsafeCell::new(false),
            data: UnsafeCell::new(data),
            waite_list: UnsafeCell::new(LinkedList::new()),
        }
    }

    // 关键方法,上锁
    #[inline(always)]
    pub fn lock(&self) -> InnerMutexGuard<T> {
        without_interrupt(|| unsafe {
            let current_task = Task::current_task();
            // 当前线程没有抢到锁则,将当前线程加入等待队列
            while self.is_locked() {
                Task::block(
                    current_task,
                    TaskState::TaskBlocked,
                    self.waite_list.get(),
                )
            }

            // 确保当前锁没有被持有
            assert!(!self.is_locked());

            // 持有锁
            *self.lock.get() = true;

            assert!(self.is_locked());

            InnerMutexGuard {
                lock: &self.lock,
                data: &mut *self.data.get(),
                waite_list: &self.waite_list,
            }
        })
    }

    pub fn is_locked(&self) -> bool {
        unsafe { *self.lock.get() }
    }
}

// 关键方法,离开作用域自动解锁
impl<'a, T: ?Sized> Drop for InnerMutexGuard<'a, T> {
    fn drop(&mut self) {
        without_interrupt(|| unsafe {
            // 确保当前锁是被锁定的
            assert!(*self.lock.get());

            // 释放锁
            *self.lock.get() = false;

            if let Some(tail_node) = (*self.waite_list.get()).end_node() {
                let first_block_task = Task::get_task(tail_node);

                if let Some(first_task) = first_block_task {
                    Task::unblock(first_task, self.waite_list.get());
                }

                sys_yield();
            }
        });
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

unsafe impl<T: ?Sized + Sync> Sync for InnerMutexGuard<'_, T> {}
unsafe impl<T: ?Sized + Send> Send for InnerMutexGuard<'_, T> {}

unsafe impl<T: ?Sized + Send> Sync for InnerMutex<T> {}
unsafe impl<T: ?Sized + Send> Send for InnerMutex<T> {}
