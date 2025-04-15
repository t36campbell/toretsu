#[cfg(test)]
mod tests {
    use std::cmp::Reverse;
    use std::collections::BinaryHeap;

    use toretsu::queue::Queue;
    use uuid::Uuid;

    #[derive(Clone, Ord, PartialEq, PartialOrd, Eq, Debug)]
    struct Int {
        v: i32,
    }

    #[test]
    fn init_queue() {
        let id = Uuid::new_v4();
        let queue: Queue<i32> = Queue::init(id, Vec::new());
        assert_eq!(queue.len(), 0);
        assert!(queue.is_empty());
    }

    #[test]
    fn new_queue() {
        let queue: Queue<i32> = Queue::new();
        assert_eq!(queue.len(), 0);
        assert!(queue.is_empty());
    }

    #[test]
    fn default_queue() {
        let queue: Queue<i32> = Queue::default();
        assert_eq!(queue.len(), 0);
        assert!(queue.is_empty());
    }

    #[test]
    fn queue_from() {
        let vec = [3, 5, 14, 2, 12, 18, 17, 11, 16, 6].map(|x| Int { v: x });
        let vector = Vec::from(vec);

        let mut queue = Queue::from(vector.clone());
        let mut heap = BinaryHeap::from(vector);
        assert_eq!(queue.len(), 10);
        assert!(!queue.is_empty());

        if let Some((a, b)) = heap.pop().zip(queue.pop()) {
            assert_eq!(a, Int { v: 18 });
            assert_eq!(a, b);
        }

        while let Some((a, b)) = heap.pop().zip(queue.pop()) {
            println!("A: {} | B:{} ", a.v, b.v);
            assert_eq!(a, b);
        }
    }

    #[test]
    fn min_queue_from() {
        let vec = [8, 9, 13, 1, 10, 7, 15, 19, 4, 20].map(|x| Reverse(Int { v: x }));
        let vector = Vec::from(vec.clone());

        let mut queue = Queue::from(vector.clone());
        let mut heap = BinaryHeap::<Reverse<Int>>::from(vector);
        assert_eq!(queue.len(), 10);
        assert!(!queue.is_empty());

        if let Some((a, b)) = heap.pop().zip(queue.pop()) {
            assert_eq!(a, Reverse(Int { v: 1 }));
            assert_eq!(a, b);
        }

        while let Some((a, b)) = heap.pop().zip(queue.pop()) {
            println!("A: {:?} | B:{:?} ", a, b);
            assert_eq!(a, b);
        }
    }

    #[test]
    fn queue_from_object() {
        #[derive(Clone, Ord, PartialEq, PartialOrd, Eq, Debug)]
        struct Obj {
            priority: i32,
        }

        let vec = [3, 5, 14, 2, 12, 18, 17, 11, 16, 6].map(|x| Box::new(Obj { priority: x }));
        let vector = Vec::from(vec);

        let mut queue = Queue::from(vector.clone());
        let mut heap = BinaryHeap::from(vector);
        assert_eq!(queue.len(), 10);
        assert!(!queue.is_empty());

        while let Some((a, b)) = heap.pop().zip(queue.pop()) {
            println!("A: {} | B:{} ", a.priority, b.priority);
            assert_eq!(a, b);
        }
    }

    #[test]
    fn queue_from_complex_object() {
        use names::{Generator, Name};
        #[derive(Clone, Ord, PartialEq, PartialOrd, Eq, Debug)]
        struct Obj {
            n: String,
            d: i32,
            r: i32,
        }

        let mut generator = Generator::with_naming(Name::Numbered);
        let vec = [3, 5, 14, 2, 12, 18, 17, 11, 16, 6].map(|x| {
            Box::new(Obj {
                n: generator.next().unwrap(),
                r: rand::random::<i32>(),
                d: x,
            })
        });
        let vector = Vec::from(vec);

        let mut queue = Queue::from(vector.clone());
        let mut heap = BinaryHeap::from(vector);
        assert_eq!(queue.len(), 10);
        assert!(!queue.is_empty());

        while let Some((a, b)) = heap.pop().zip(queue.pop()) {
            println!("A: {} {} {} | B: {} {} {}", a.n, a.d, a.r, b.n, b.d, b.r);
            assert_eq!(a, b);
        }
    }

    #[test]
    fn queue_from_object_custom_ord() {
        use names::{Generator, Name};
        use rand::random;

        #[derive(Clone, PartialEq, Eq, Debug)]
        struct Obj {
            n: String,
            d: i32,
            r: i32,
        }

        impl Ord for Obj {
            fn cmp(&self, other: &Self) -> std::cmp::Ordering {
                self.d.cmp(&other.d)
            }
        }

        impl PartialOrd for Obj {
            fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
                Some(self.cmp(other))
            }
        }

        let mut generator = Generator::with_naming(Name::Numbered);
        let vec = [3, 5, 14, 2, 12, 18, 17, 11, 16, 6].map(|x| {
            Box::new(Obj {
                n: generator.next().unwrap(),
                r: random::<i32>(),
                d: x,
            })
        });
        let vector = Vec::from(vec);

        let mut queue = Queue::from(vector.clone());
        assert_eq!(queue.len(), 10);
        assert!(!queue.is_empty());

        let mut heap = BinaryHeap::from(vector);
        while let Some((a, b)) = heap.pop().zip(queue.pop()) {
            println!("A: {} {} {} | B: {} {} {}", a.n, a.d, a.r, b.n, b.d, b.r);
            assert_eq!(a, b);
        }
    }

    #[test]
    fn queue_with_capacity() {
        let queue: Queue<i32> = Queue::with_capacity(10);
        assert_eq!(queue.capacity(), 10);
        assert!(queue.is_empty());
    }

    #[test]
    fn queue_peek() {
        let vec = [3, 5, 14, 2, 12, 18, 17, 11, 16, 6].map(|x| Int { v: x });
        let vector = Vec::from(vec);

        let queue = Queue::from(vector);
        assert_eq!(queue.len(), 10);

        let peeked = queue.peek().unwrap();
        assert!(queue.len() == 10);
        assert_eq!(peeked.v, 18);
    }

    #[test]
    fn queue_into_sorted_vec() {
        let vec = [3, 5, 14, 2, 12, 18, 17, 11, 16, 6].map(|x| Int { v: x });
        let vector = Vec::from(vec);

        let queue = Queue::from(vector);
        let sorted = queue.into_sorted_vec();

        assert_eq!(
            sorted,
            [
                Int { v: 2 },
                Int { v: 3 },
                Int { v: 5 },
                Int { v: 6 },
                Int { v: 11 },
                Int { v: 12 },
                Int { v: 14 },
                Int { v: 16 },
                Int { v: 17 },
                Int { v: 18 }
            ]
        );
    }
}
