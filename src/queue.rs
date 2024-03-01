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

    pub fn push(&mut self, value: T) {
        self.heap.push(value)
    }

    pub fn pop(&mut self) -> Option<T> {
        self.heap.pop()
    }

    pub fn peek(&self) -> Option<&T> {
        self.heap.peek()
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
}
