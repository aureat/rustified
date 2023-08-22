use std::marker::PhantomData;
use std::mem::ManuallyDrop;
use std::ops::{Deref, DerefMut};
use std::{ptr, slice};

use super::into_iter::IntoIter;
use super::raw_vec::RawVec;

pub struct Vec<T> {
    buf: RawVec<T>,
    len: usize,
}

impl<T> Vec<T> {
    pub const fn new() -> Self {
        Self {
            buf: RawVec::NEW,
            len: 0,
        }
    }

    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            buf: RawVec::with_capacity(capacity),
            len: 0,
        }
    }

    pub fn with_capacity_zeroed(capacity: usize) -> Self {
        Self {
            buf: RawVec::with_capacity_zeroed(capacity),
            len: 0,
        }
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.len
    }

    #[inline]
    pub fn capacity(&self) -> usize {
        self.buf.capacity()
    }

    #[inline]
    pub fn as_mut_ptr(&mut self) -> *mut T {
        self.buf.ptr()
    }

    #[inline]
    pub fn as_ptr(&self) -> *const T {
        self.buf.ptr()
    }

    #[inline]
    pub(crate) unsafe fn set_len(&mut self, len: usize) {
        assert!(len <= self.capacity());
        self.len = len;
    }

    pub fn push(&mut self, value: T) {
        // if length reached capacity, request an additional space of 1
        if self.len == self.capacity() {
            self.buf.reserve_for_push(self.len);
        }

        // p.add(capacity) is the end of the last byte of allocated space
        // you can write an element T, at most, at location p.add(capacity - 1)
        // SAFETY: offset is valid, since len < capacity, so len <= (capacity - 1) < capacity
        unsafe {
            let end = self.as_mut_ptr().add(self.len);
            ptr::write(end, value);
            self.len += 1;
        }
    }

    pub fn pop(&mut self) -> Option<T> {
        if self.len == 0 {
            return None;
        }

        // p.add(capacity) is the end of the last byte of allocated space
        // you can read an element T, at most, at location p.add(capacity - 1)
        // SAFETY: offset is valid, since len <= capacity, so (len - 1) <= (capacity - 1) < capacity
        self.len -= 1;
        unsafe {
            let end = self.as_ptr().add(self.len);
            Some(ptr::read(end))
        }
    }

    pub fn insert(&mut self, index: usize, element: T) {
        // check bounds
        let len = self.len();
        assert!(index <= len, "index out of bounds");

        // reserve space for the new element
        if len == self.capacity() {
            self.buf.reserve(len, 1);
        }

        // p.add(capacity) is the end of the last byte of allocated space
        // you can write an element T, at most, at location p.add(capacity - 1)
        // SAFETY: offset is valid, since index <= len <= (capacity - 1) < capacity
        unsafe {
            {
                let p = self.as_mut_ptr().add(index);
                // here, index < len, so (index + 1) <= len <= (capacity - 1) < capacity
                if index < len {
                    ptr::copy(p, p.add(1), len - index);
                }
                ptr::write(p, element);
            }
            self.set_len(len + 1);
        }
    }

    pub fn remove(&mut self, index: usize) -> T {
        // check bounds
        let len = self.len();
        assert!(index < len, "index out of bounds");

        // p.add(capacity) is the end of the last byte of allocated space
        // you can write an element T, at most, at location p.add(capacity - 1)
        // SAFETY: offset is valid, since index < len <= (capacity - 1) < capacity
        unsafe {
            let value: T;
            {
                let p = self.as_mut_ptr().add(index);
                value = ptr::read(p);
                ptr::copy(p.add(1), p, len - index - 1);
            }
            self.set_len(len - 1);
            value
        }
    }

    pub fn swap_remove(&mut self, index: usize) -> T {
        // check bounds
        let len = self.len();
        assert!(index < len, "index out of bounds");

        unsafe {
            let value = ptr::read(self.as_ptr().add(index));
            let base_ptr = self.as_mut_ptr();
            ptr::copy(base_ptr.add(len - 1), base_ptr.add(index), 1);
            self.set_len(len - 1);
            value
        }
    }
}

impl<T> Deref for Vec<T> {
    type Target = [T];
    fn deref(&self) -> &[T] {
        unsafe { slice::from_raw_parts(self.as_ptr(), self.len()) }
    }
}

impl<T> DerefMut for Vec<T> {
    fn deref_mut(&mut self) -> &mut [T] {
        unsafe { slice::from_raw_parts_mut(self.as_mut_ptr(), self.len()) }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct ZST;
    const UMAX: usize = usize::MAX;

    #[test]
    fn basic_test() {
        let mut v = Vec::<usize>::new();
        assert_eq!(v.capacity(), 0);
        v.push(0);
        assert_eq!(v.capacity(), 4);
        v.push(1);
        assert_eq!(v.len(), 2);
    }

    #[test]
    fn zst_test() {
        let mut v = Vec::<ZST>::new();
        assert_eq!((v.capacity(), v.len()), (UMAX, 0));
        v.push(ZST);
        assert_eq!((v.capacity(), v.len()), (UMAX, 1));
        for _ in 0..100 {
            v.push(ZST);
        }
        assert_eq!((v.capacity(), v.len()), (UMAX, 101));
    }

    #[test]
    #[should_panic]
    fn zst_overflow() {
        let mut v = Vec::<ZST>::new();
        unsafe { v.set_len(UMAX) };
        assert_eq!((v.capacity(), v.len()), (UMAX, UMAX));
        v.push(ZST);
    }

    #[test]
    fn cap_test() {
        let mut v = Vec::<usize>::new();
        for num in 0..16 {
            v.push(num);
        }
        assert_eq!((v.capacity(), v.len()), (16, 16));
        for num in 16..100 {
            v.push(num);
        }
        assert_eq!((v.capacity(), v.len()), (128, 100));
    }
}
