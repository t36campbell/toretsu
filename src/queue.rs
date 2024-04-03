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

    pub fn new_in(alloc: A) -> BinaryHeap<T, A> {
        let id = Uuid::new_v4();
        Self {
            id,
            heap: BinaryHeap::new_in(alloc)
        }
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
    pub fn with_capacity_in(capacity: usize, alloc: A) -> BinaryHeap<T, A> {
        let id = Uuid::new_v4();
        Self {
            id,
            heap: BinaryHeap::with_capacity_in(capacity, alloc),
        }
    }

    pub fn capacity(&self) -> usize {
        self.heap.capacity()
    }

    pub fn push(&mut self, value: T) {
        self.heap.push(value)
    }

    pub fn extend(&mut self, iter: Vec<T>) {
        self.heap.extend(iter)
    }
    
    pub fn append(&mut self, other: &mut Self) {
        self.heap.append(other)
    }

    pub fn pop(&mut self) -> Option<T> {
        self.heap.pop()
    }

    pub fn peek(&self) -> Option<&T> {
        self.heap.peek()
    }
    
    pub fn peek_mut(&self) -> Option<PeekMut<'_, T, A>> { {
        self.heap.peek_mut()
    }

    pub fn into_sorted_vec(self) -> Vec<T> {
        self.heap.into_sorted_vec()
    }

    pub fn len(&self) -> usize {
        self.heap.len()
    }

    pub fn is_empty(&self) -> bool {
        self.heap.is_empty()
    }

    pub fn drain(&mut self) -> std::collections::binary_heap::Drain<'_, T> {
        self.heap.drain()
    }

    pub fn drain_sorted(&mut self) {
        todo!("this needs an unstable feature")
    }
    
    pub fn retain(&mut self) {
        todo!("this needs feature: binary_heap_retain")
    }

    pub fn try_reserve_exact(&mut self) {
        todo!("this needs feature: try_reserve_2")
    }

    pub fn try_reserve(&mut self) {
        todo!("this needs feature: try_reserve_2")
    }
    
    pub fn shrink_to(&mut self) {
        todo!("this needs feature: shrink_to")
    }
    
    pub fn as_slice(&mut self) {
        todo!("this needs feature: binary_heap_as_slice")
    }

}
