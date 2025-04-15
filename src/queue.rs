use std::collections::BinaryHeap;

use uuid::Uuid;

pub struct Queue<T> {
    pub id: Uuid,
    heap: BinaryHeap<T>,
}

impl<T: Ord> Default for Queue<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T: Ord> Iterator for Queue<T> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        self.pop()
    }
}

impl<T: Ord> Queue<T> {
    pub fn init(id: Uuid, vec: Vec<T>) -> Self {
        Self {
            id,
            heap: BinaryHeap::from(vec),
        }
    }

    pub fn new() -> Self {
        let id = Uuid::new_v4();
        let vec = Vec::new();

        Self::init(id, vec)
    }

    pub fn from(vec: Vec<T>) -> Self {
        let id = Uuid::new_v4();

        Self::init(id, vec)
    }

    pub fn with_capacity(capacity: usize) -> Self {
        let id = Uuid::new_v4();
        Self {
            id,
            heap: BinaryHeap::with_capacity(capacity),
        }
    }

    pub fn capacity(&self) -> usize {
        self.heap.capacity()
    }

    pub fn len(&self) -> usize {
        self.heap.len()
    }

    pub fn is_empty(&self) -> bool {
        self.heap.is_empty()
    }

    pub fn clear(&mut self) {
        self.heap.clear()
    }

    pub fn push(&mut self, value: T) {
        self.heap.push(value)
    }

    pub fn pop(&mut self) -> Option<T> {
        self.heap.pop()
    }

    pub fn peek(&self) -> Option<&T> {
        self.heap.peek()
    }

    pub fn peek_mut(&mut self) -> Option<std::collections::binary_heap::PeekMut<'_, T>> {
        self.heap.peek_mut()
    }

    pub fn drain(&mut self) -> std::collections::binary_heap::Drain<'_, T> {
        self.heap.drain()
    }

    pub fn drain_sorted(&mut self) -> Vec<T> {
        let mut vec = Vec::with_capacity(self.len());
        for val in self {
            vec.push(val);
        }

        vec
    }

    pub fn into_vec(self) -> Vec<T> {
        self.heap.into_vec()
    }

    pub fn into_sorted_vec(self) -> Vec<T> {
        self.heap.into_sorted_vec()
    }

    pub fn extend<I: IntoIterator<Item = T>>(&mut self, iter: I) {
        self.heap.extend(iter)
    }

    pub fn append(&mut self, other: &mut Self) {
        self.heap.append(&mut other.heap)
    }

    pub fn as_slice(&self) -> &[T] {
        self.heap.as_slice()
    }

    pub fn reserve(&mut self, additional: usize) {
        self.heap.reserve(additional)
    }

    pub fn shrink_to(&mut self, min_capacity: usize) {
        self.heap.shrink_to(min_capacity)
    }

    pub fn shrink_to_fit(&mut self) {
        self.heap.shrink_to_fit()
    }

    pub fn retain<F>(&mut self, f: F)
    where
        F: FnMut(&T) -> bool,
    {
        self.heap.retain(f)
    }

    pub fn reserve_exact(&mut self, additional: usize) {
        self.heap.reserve_exact(additional)
    }

    pub fn try_reserve(
        &mut self,
        additional: usize,
    ) -> Result<(), std::collections::TryReserveError> {
        self.heap.try_reserve(additional)
    }

    pub fn try_reserve_exact(
        &mut self,
        additional: usize,
    ) -> Result<(), std::collections::TryReserveError> {
        self.heap.try_reserve_exact(additional)
    }
}
