use alloc::alloc::Global;
use alloc::boxed::Box;
use core::alloc::Allocator;
use core::marker::PhantomData;
use core::ptr::{NonNull, Unique};

pub struct LinkedList<T, A: Allocator = Global> {
    head: Option<NonNull<Node<T>>>,
    tail: Option<NonNull<Node<T>>>,
    len: usize,
    alloc: A,
    marker: PhantomData<Box<Node<T>, A>>,
}

#[derive(Clone, Eq, PartialEq, Debug)]
pub struct Node<T> {
    pub next: Option<NonNull<Node<T>>>,
    pub prev: Option<NonNull<Node<T>>>,
    pub element: T,
}

impl<T> Node<T> {
    fn new(element: T) -> Self {
        Node {
            next: None,
            prev: None,
            element,
        }
    }

    #[allow(clippy::boxed_local)]
    fn into_element<A: Allocator>(self: Box<Self, A>) -> T {
        self.element
    }
}

impl<T, A: Allocator> LinkedList<T, A> {
    #[inline]
    pub unsafe fn push_front_node(&mut self, node: Unique<Node<T>>) {
        unsafe {
            (*node.as_ptr()).next = self.head;
            (*node.as_ptr()).prev = None;
            let node = Some(NonNull::from(node));

            match self.head {
                None => self.tail = node,
                // Not creating new mutable (unique!) references overlapping `element`.
                Some(head) => (*head.as_ptr()).prev = node,
            }

            self.head = node;
            self.len += 1;
        }
    }

    #[inline]
    pub fn pop_front_node(&mut self) -> Option<Box<Node<T>, &A>> {
        self.head.map(|node| unsafe {
            let node = Box::from_raw_in(node.as_ptr(), &self.alloc);
            self.head = node.next;

            match self.head {
                None => self.tail = None,
                // Not creating new mutable (unique!) references overlapping `element`.
                Some(head) => (*head.as_ptr()).prev = None,
            }

            self.len -= 1;
            node
        })
    }

    #[inline]
    pub unsafe fn push_back_node(&mut self, node: Unique<Node<T>>) {
        unsafe {
            (*node.as_ptr()).next = None;
            (*node.as_ptr()).prev = self.tail;
            let node = Some(NonNull::from(node));

            match self.tail {
                None => self.head = node,
                Some(tail) => (*tail.as_ptr()).next = node,
            }

            self.tail = node;
            self.len += 1;
        }
    }

    #[inline]
    pub fn pop_back_node(&mut self) -> Option<Box<Node<T>, &A>> {
        self.tail.map(|node| unsafe {
            let node = Box::from_raw_in(node.as_ptr(), &self.alloc);
            self.tail = node.prev;

            match self.tail {
                None => self.head = None,
                Some(tail) => (*tail.as_ptr()).next = None,
            }

            self.len -= 1;
            node
        })
    }

    #[inline]
    pub unsafe fn unlink_node(&mut self, mut node: NonNull<Node<T>>) {
        let node = unsafe { node.as_mut() }; // this one is ours now, we can create an &mut.

        match node.prev {
            Some(prev) => unsafe { (*prev.as_ptr()).next = node.next },
            // this node is the head node
            None => self.head = node.next,
        };

        match node.next {
            Some(next) => unsafe { (*next.as_ptr()).prev = node.prev },
            // this node is the tail node
            None => self.tail = node.prev,
        };

        self.len -= 1;
    }

    // 在指定元素前插入新元素
    #[inline]
    pub unsafe fn push_back_anchor_node(
        &mut self,
        mut existing_node: NonNull<Node<T>>,
        mut new_node: NonNull<Node<T>>,
    ) {
        let existing_node_next = existing_node.as_mut().next.take();

        new_node.as_mut().prev = Some(existing_node);
        new_node.as_mut().next = existing_node_next;

        existing_node.as_mut().next = Some(new_node);
        if let Some(mut existing_node_next) = existing_node.as_mut().next {
            existing_node_next.as_mut().prev = Some(new_node);
        }

        if self.tail.is_none() || self.tail == Some(existing_node) {
            self.tail = Some(new_node);
        }

        self.len += 1;
    }

    // 搜索函数
    pub fn find_node<F>(&self, mut condition: F) -> Option<NonNull<Node<T>>>
    where
        F: FnMut(NonNull<Node<T>>) -> bool,
    {
        let mut current_node = self.head;

        while let Some(mut node) = current_node {
            if condition(node) {
                return Some(node);
            }
            current_node = unsafe { node.as_mut().next };
        }

        None
    }

    // 头节点
    pub fn front_node(&self) -> Option<NonNull<Node<T>>> {
        self.head
    }

    // 尾节点
    pub fn end_node(&self) -> Option<NonNull<Node<T>>> {
        self.tail
    }
}

impl<T> LinkedList<T> {
    #[inline]
    #[must_use]
    pub const fn new() -> Self {
        LinkedList {
            head: None,
            tail: None,
            len: 0,
            alloc: Global,
            marker: PhantomData,
        }
    }

    #[inline]
    #[must_use]
    pub fn front(&self) -> Option<&T> {
        unsafe { self.head.as_ref().map(|node| &node.as_ref().element) }
    }

    #[inline]
    #[must_use]
    pub fn front_mut(&mut self) -> Option<&mut T> {
        unsafe { self.head.as_mut().map(|node| &mut node.as_mut().element) }
    }

    #[inline]
    #[must_use]
    pub fn back(&self) -> Option<&T> {
        unsafe { self.tail.as_ref().map(|node| &node.as_ref().element) }
    }

    #[inline]
    pub fn back_mut(&mut self) -> Option<&mut T> {
        unsafe { self.tail.as_mut().map(|node| &mut node.as_mut().element) }
    }

    pub fn push_front(&mut self, elt: T) {
        let node = Box::new_in(Node::new(elt), &self.alloc);
        let node_ptr = Unique::from(Box::leak(node));
        unsafe {
            self.push_front_node(node_ptr);
        }
    }

    pub fn pop_front(&mut self) -> Option<T> {
        self.pop_front_node().map(Node::into_element)
    }

    #[inline]
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.head.is_none()
    }
}

unsafe impl<T: Send, A: Allocator + Send> Send for LinkedList<T, A> {}

unsafe impl<T: Sync, A: Allocator + Sync> Sync for LinkedList<T, A> {}

unsafe impl<T: Send> Send for Node<T> {}

unsafe impl<T: Sync> Sync for Node<T> {}
