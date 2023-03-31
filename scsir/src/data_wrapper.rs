#![allow(dead_code)]

use std::{
    alloc::Layout,
    borrow::{Borrow, BorrowMut},
    fmt::Debug,
    mem::{self, MaybeUninit},
    ops::{Deref, DerefMut},
    ptr, slice,
};

pub type AnyType = [u8; 0];

pub struct FlexibleStruct<Body, Element> {
    length: usize,
    capacity: usize,
    ptr: *mut Raw<Body, Element>,
}

impl<Body, Element> FlexibleStruct<Body, Element> {
    pub fn with_body_capacity(body: Body, capacity: usize) -> Self {
        let size = mem::size_of::<Body>() + mem::size_of::<Element>() * capacity;
        let layout = Layout::from_size_align(size, mem::align_of::<Body>()).unwrap();
        let ptr = unsafe { std::alloc::alloc_zeroed(layout) as *mut Raw<Body, Element> };

        unsafe {
            ptr::write_unaligned(ptr::addr_of_mut!((*ptr).body) as *mut Body, body);
        }

        Self {
            length: 0,
            capacity,
            ptr,
        }
    }

    pub fn get_body_maybe_uninit(&self) -> MaybeUninit<Body> {
        unsafe {
            MaybeUninit::new(ptr::read_unaligned(
                ptr::addr_of!((*self.ptr).body) as *const Body
            ))
        }
    }

    pub fn set_body(&mut self, value: Body) {
        unsafe {
            ptr::write_unaligned(ptr::addr_of_mut!((*self.ptr).body) as *mut Body, value);
        }
    }

    pub fn push(&mut self, value: Element) {
        let new_capacity = if self.length + 1 > self.capacity {
            self.capacity * 2 + 1
        } else {
            self.capacity
        };
        self.try_grow_to(new_capacity);

        unsafe {
            let base = std::ptr::addr_of_mut!((*self.ptr).array) as *mut Element;
            let target = base.add(self.length);

            ptr::write_unaligned(target.cast(), value);

            self.length += 1;
        };
    }

    pub fn pop(&mut self) -> Option<Element> {
        if self.length == 0 {
            None
        } else {
            unsafe {
                self.length -= 1;

                let base = std::ptr::addr_of!((*self.ptr).array) as *const Element;
                let target = base.add(self.length);

                Some(ptr::read_unaligned(target.cast()))
            }
        }
    }

    pub fn clear(&mut self) {
        if std::mem::needs_drop::<Element>() {
            while self.pop().is_some() {}
        } else {
            self.length = 0;
        }
    }

    pub unsafe fn body_as_ref(&self) -> &Body {
        &*ptr::addr_of!((*self.ptr).body)
    }

    pub unsafe fn body_as_mut(&mut self) -> &mut Body {
        &mut *ptr::addr_of_mut!((*self.ptr).body)
    }

    pub unsafe fn elements_as_slice(&self) -> &[Element] {
        slice::from_raw_parts(ptr::addr_of!((*self.ptr).array).cast(), self.length)
    }

    pub unsafe fn elements_as_mut_slice(&mut self) -> &mut [Element] {
        slice::from_raw_parts_mut(ptr::addr_of_mut!((*self.ptr).array).cast(), self.length)
    }

    pub fn total_size(&self) -> usize {
        mem::size_of::<Body>() + mem::size_of::<Element>() * self.length
    }

    pub fn as_bytes(&self) -> &[u8] {
        unsafe { slice::from_raw_parts(self.ptr.cast(), self.total_size()) }
    }

    pub fn as_bytes_mut(&mut self) -> &mut [u8] {
        unsafe { slice::from_raw_parts_mut(self.ptr.cast(), self.total_size()) }
    }

    pub fn length(&self) -> usize {
        self.length
    }

    pub fn capacity(&self) -> usize {
        self.capacity
    }

    pub fn iter_maybe_uninit(&self) -> MaybeUninitIter<Body, Element> {
        MaybeUninitIter {
            this: self,
            index: 0,
        }
    }

    pub fn get_element_maybe_uninit(&self, index: usize) -> Option<MaybeUninit<Element>> {
        if index >= self.length {
            None
        } else {
            unsafe {
                let base = std::ptr::addr_of!((*self.ptr).array) as *const Element;
                let target = base.add(index);

                Some(MaybeUninit::new(ptr::read_unaligned(target)))
            }
        }
    }

    fn try_grow_to(&mut self, new_capacity: usize) {
        if new_capacity <= self.capacity {
            return;
        }

        let new_size = mem::size_of::<Body>() + mem::size_of::<Element>() * new_capacity;
        let layout = Layout::from_size_align(new_size, mem::align_of::<Body>()).unwrap();
        let memory = unsafe {
            std::alloc::realloc(self.ptr.cast(), layout, new_size) as *mut Raw<Body, Element>
        };

        if memory.is_null() {
            panic!("Out of memory!");
        }

        self.ptr = memory;
        self.capacity = new_capacity;
    }
}

impl<Body: Clone, Element> FlexibleStruct<Body, Element> {
    pub fn get_body(&self) -> Body {
        unsafe { self.get_body_maybe_uninit().assume_init_ref().clone() }
    }
}

impl<Body: Default, Element> FlexibleStruct<Body, Element> {
    pub fn new() -> Self {
        Self::with_capacity(0)
    }

    pub fn with_capacity(capacity: usize) -> Self {
        Self::with_body_capacity(Body::default(), capacity)
    }
}

impl<Body, Element: Clone> FlexibleStruct<Body, Element> {
    pub fn iter_clone(&self) -> CloneIter<Body, Element> {
        CloneIter {
            maybe_uninit_iter: self.iter_maybe_uninit(),
        }
    }

    pub fn get_element(&self, index: usize) -> Option<Element> {
        unsafe {
            self.get_element_maybe_uninit(index)
                .map(|e| e.assume_init_ref().clone())
        }
    }
}

impl<Body, Element: Copy> FlexibleStruct<Body, Element> {
    pub unsafe fn with_body_length(body: Body, length: usize) -> Self {
        let mut temp = Self::with_body_capacity(body, length);

        temp.set_length(length);

        temp
    }

    // No initialization
    pub unsafe fn set_length(&mut self, length: usize) {
        self.try_grow_to(length);
        self.length = length;
    }
}

impl<Body: Copy, Element: Copy> FlexibleStruct<Body, Element> {
    pub unsafe fn with_length(length: usize) -> Self {
        Self::with_body_length(mem::zeroed(), length)
    }
}

impl<B, E> Borrow<AnyType> for FlexibleStruct<B, E> {
    fn borrow(&self) -> &AnyType {
        unsafe { &*self.as_bytes().as_ptr().cast() }
    }
}

impl<B, E> BorrowMut<AnyType> for FlexibleStruct<B, E> {
    fn borrow_mut(&mut self) -> &mut AnyType {
        unsafe { &mut *self.as_bytes_mut().as_mut_ptr().cast() }
    }
}

impl<B: Clone, E: Clone> Clone for FlexibleStruct<B, E> {
    fn clone(&self) -> Self {
        let mut new_struct = Self::with_body_capacity(self.get_body(), self.length);

        for item in self.iter_clone() {
            new_struct.push(item);
        }

        new_struct
    }
}

impl<B: Debug, E: Debug> Debug for FlexibleStruct<B, E> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        unsafe {
            f.debug_struct("FlexibleStruct")
                .field("length", &self.length)
                .field("capacity", &self.capacity)
                .field("body", self.get_body_maybe_uninit().assume_init_ref())
                .field("elements", &self.iter_maybe_uninit())
                .finish()
        }
    }
}

impl<Body: Default, Element> Default for FlexibleStruct<Body, Element> {
    fn default() -> Self {
        Self::new()
    }
}

impl<B, E> Drop for FlexibleStruct<B, E> {
    fn drop(&mut self) {
        let layout = Layout::from_size_align(
            mem::size_of::<B>() + self.capacity * mem::size_of::<E>(),
            mem::align_of::<B>(),
        )
        .unwrap();

        self.clear();

        unsafe {
            self.get_body_maybe_uninit().assume_init();
            std::alloc::dealloc(self.ptr.cast(), layout);
        }
    }
}

pub struct MaybeUninitIter<'a, B, E> {
    this: &'a FlexibleStruct<B, E>,
    index: usize,
}

impl<'a, B, E: Debug> Debug for MaybeUninitIter<'a, B, E> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut list = f.debug_list();

        for item in self.this.iter_maybe_uninit() {
            list.entry(unsafe { item.assume_init_ref() });
        }

        list.finish()
    }
}

impl<'a, B, E> Iterator for MaybeUninitIter<'a, B, E> {
    type Item = MaybeUninit<E>;

    fn next(&mut self) -> Option<Self::Item> {
        let result = if self.index == self.this.length {
            None
        } else {
            self.this.get_element_maybe_uninit(self.index)
        };

        self.index += 1;

        result
    }
}

#[derive(Debug)]
pub struct CloneIter<'a, B, E: Clone> {
    maybe_uninit_iter: MaybeUninitIter<'a, B, E>,
}

impl<'a, B, E: Clone> Iterator for CloneIter<'a, B, E> {
    type Item = E;

    fn next(&mut self) -> Option<Self::Item> {
        self.maybe_uninit_iter
            .next()
            .map(|e| unsafe { e.assume_init_ref().clone() })
    }
}

#[repr(packed)]
struct Raw<Body, Element> {
    body: Body,
    array: [Element; 0],
}

#[repr(transparent)]
#[derive(Clone, Debug, Default)]
pub(crate) struct VecBufferWrapper(pub Vec<u8>);

impl VecBufferWrapper {
    pub unsafe fn with_len(len: usize) -> Self {
        let mut temp = Self(Vec::with_capacity(len));

        temp.set_len(len);

        temp
    }
}

impl Borrow<AnyType> for VecBufferWrapper {
    fn borrow(&self) -> &AnyType {
        unsafe { &*self.0.as_ptr().cast() }
    }
}

impl BorrowMut<AnyType> for VecBufferWrapper {
    fn borrow_mut(&mut self) -> &mut AnyType {
        unsafe { &mut *self.0.as_mut_ptr().cast() }
    }
}

impl Deref for VecBufferWrapper {
    type Target = Vec<u8>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for VecBufferWrapper {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl From<Vec<u8>> for VecBufferWrapper {
    fn from(value: Vec<u8>) -> Self {
        Self(value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn layout_test() {
        let body: [u8; 10] = [0, 1, 2, 3, 4, 5, 6, 7, 8, 9];
        let element1: [u8; 5] = [9, 8, 7, 6, 5];
        let element2: [u8; 5] = [4, 3, 2, 1, 0];

        let mut temp = FlexibleStruct::new();
        temp.set_body(body);

        assert_eq!(
            temp.total_size(),
            body.len(),
            concat!("Size of FlexibleStruct with 0 element")
        );

        temp.push(element1);

        assert_eq!(
            temp.total_size(),
            body.len() + element1.len(),
            concat!("Size of FlexibleStruct with 1 element")
        );

        temp.push(element2);

        assert_eq!(
            temp.total_size(),
            body.len() + element1.len() + element2.len(),
            concat!("Size of FlexibleStruct with 2 element")
        );

        assert_eq!(
            &temp.as_bytes()[..body.len()],
            &body,
            concat!("body comparation")
        );

        assert_eq!(
            &temp.as_bytes()[body.len()..body.len() + element1.len()],
            &element1,
            concat!("element1 comparation")
        );

        assert_eq!(
            &temp.as_bytes()
                [body.len() + element1.len()..body.len() + element1.len() + element2.len()],
            &element2,
            concat!("element2 comparation")
        );

        assert_eq!(
            &temp.as_bytes(),
            &temp.clone().as_bytes(),
            concat!("clone comparation")
        );
    }

    struct Dropper {
        marker: *mut bool,
    }
    impl Drop for Dropper {
        fn drop(&mut self) {
            unsafe { *self.marker = true };
        }
    }

    #[test]
    fn drop_test() {
        let mut body_marker = false;
        let mut marker_1 = false;
        let mut marker_2 = false;

        let mut tester = FlexibleStruct::<Dropper, Dropper>::with_body_capacity(
            Dropper {
                marker: &mut body_marker,
            },
            0,
        );

        tester.push(Dropper {
            marker: &mut marker_1,
        });
        tester.push(Dropper {
            marker: &mut marker_2,
        });

        tester.pop();

        assert_eq!(marker_2, true, "marker 2 dropped");

        tester.clear();

        assert_eq!(marker_1, true, "marker 1 dropped");

        drop(tester);

        assert_eq!(body_marker, true, "body marker dropped");
    }
}
