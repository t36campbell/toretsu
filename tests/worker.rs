#[cfg(test)]
mod tests {
    use toretsu::queue::Queue;
    use toretsu::task::Task;
    use toretsu::worker::Worker;
    use uuid::Uuid;

    #[derive(Clone, Copy, Eq, PartialEq, Ord, PartialOrd)]
    pub struct Job<U, V, W> {
        kwargs: U,
        args: V,
        result: W,
        function: fn(U, V) -> W,
    }

    impl<U, V, W> Job<U, V, W>
    where
        U: Copy,
        V: Copy + Default,
        W: Default,
    {
        pub fn init(
            kwargs: U,
            function: fn(U, V) -> W,
            args: Option<V>,
            result: Option<W>,
        ) -> Self {
            let args = match args {
                Some(args) => args,
                None => V::default(),
            };

            let result = match result {
                Some(result) => result,
                None => W::default(),
            };

            Self {
                kwargs,
                args,
                result,
                function,
            }
        }

        pub fn new(kwargs: U, function: fn(U, V) -> W) -> Self {
            Self::init(kwargs, function, None, None)
        }

        #[allow(dead_code)]
        pub fn new_with_args(kwargs: U, function: fn(U, V) -> W, args: V) -> Self {
            Self::init(kwargs, function, Some(args), None)
        }
    }

    impl<U, V, W> Task for Job<U, V, W>
    where
        U: Copy,
        V: Copy + Default,
        W: Default,
    {
        fn process(&mut self) {
            let result = (self.function)(self.kwargs, self.args);
            self.result = result;
        }
    }

    #[derive(Clone, Copy, Eq, PartialEq, Ord, PartialOrd)]
    struct Options {}

    #[test]
    fn test_init() {
        let id = Uuid::new_v4();
        let channel = "test".to_string();
        let vec: [Job<i32, Option<Options>, ()>; 10] =
            [3, 5, 14, 2, 12, 18, 17, 11, 16, 6].map(|x| Job::new(x, callback));

        let vector = Vec::from(vec);
        let queue = Queue::from(vector);
        let worker = Worker::init(Some(id), Some(channel), Some(queue));
        assert_eq!(worker.id, id);
        assert_eq!(worker.channel, "test");
    }

    fn callback<T: std::fmt::Debug, U>(item: T, _options: Option<U>) {
        println!("Processed item {:?}", item)
    }

    #[test]
    fn test_worker_from() {
        let vec: [Job<i32, Option<Options>, ()>; 10] =
            [3, 5, 14, 2, 12, 18, 17, 11, 16, 6].map(|x| Job::new(x, callback));

        let vector = Vec::from(vec);
        let worker = Worker::from(vector);
        assert_eq!(worker.queue.len(), 10);
    }

    #[test]
    fn test_worker_start() {
        let vec: [Job<i32, Option<Options>, ()>; 10] =
            [8, 9, 13, 1, 10, 7, 15, 19, 4, 20].map(|x| Job::new(x, callback));

        let vector = Vec::from(vec);
        let mut worker = Worker::from(vector);
        assert_eq!(worker.queue.len(), 10);

        worker.clock_in();
        assert!(worker.queue.is_empty());

        worker.clock_out();
        assert!(!worker.active);
    }

    #[test]
    fn test_worker_loop() {
        let vec: [Job<i32, Option<Options>, ()>; 10] =
            [8, 9, 13, 1, 10, 7, 15, 19, 4, 20].map(|x| Job::new(x, callback));

        let vector = Vec::from(vec);
        let mut worker = Worker::from(vector);
        assert_eq!(worker.queue.len(), 10);

        worker.clock_in();
        assert!(worker.queue.is_empty());

        let w = [3, 5, 14, 2, 12, 18, 17, 11, 16, 6].map(|x| Job::new(x, callback));

        let work = Vec::from(w);
        worker.assign_many(work);
        assert!(worker.queue.is_empty());

        worker.assign_one(Job::new(21, callback));
        assert!(worker.queue.is_empty());

        worker.clock_out();
        assert!(!worker.active);
    }
}
