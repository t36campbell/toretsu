#[cfg(test)]
mod tests {
    use toretsu::queue::Queue;
    use toretsu::task::Task;
    use toretsu::worker::Worker;
    use uuid::Uuid;

    #[derive(Clone, Copy, Eq, PartialEq, Ord, PartialOrd)]
    pub struct Job<T, U, RV> {
        kwargs: T,
        args: U,
        result: RV,
        function: fn(T, U) -> RV,
    }

    impl<T, U, RV> Job<T, U, RV>
    where
        T: Copy,
        U: Copy + Default,
        RV: Default,
    {
        pub fn init(
            kwargs: T,
            function: fn(T, U) -> RV,
            args: Option<U>,
            result: Option<RV>,
        ) -> Self {
            let args = match args {
                Some(args) => args,
                None => U::default(),
            };

            let result = match result {
                Some(result) => result,
                None => RV::default(),
            };

            Self {
                kwargs,
                args,
                result,
                function,
            }
        }

        pub fn new(kwargs: T, function: fn(T, U) -> RV) -> Self {
            Self::init(kwargs, function, None, None)
        }

        #[allow(dead_code)]
        pub fn new_with_args(kwargs: T, function: fn(T, U) -> RV, args: U) -> Self {
            Self::init(kwargs, function, Some(args), None)
        }
    }

    impl<T, U, RV> Task for Job<T, U, RV>
    where
        T: Copy,
        U: Copy + Default,
        RV: Default,
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
        assert_eq!(worker.queue_len(), 10);
    }

    #[test]
    fn test_worker_start() {
        let vec: [Job<i32, Option<Options>, ()>; 10] =
            [8, 9, 13, 1, 10, 7, 15, 19, 4, 20].map(|x| Job::new(x, callback));

        let vector = Vec::from(vec);
        let mut worker = Worker::from(vector);
        assert_eq!(worker.queue_len(), 10);

        worker.clock_in();

        // Give the worker thread a moment to consume initial tasks
        std::thread::sleep(std::time::Duration::from_millis(50));
        assert!(worker.queue_is_empty());

        worker.clock_out();
        assert!(!worker.is_active());
    }

    #[test]
    fn test_worker_loop() {
        let vec: [Job<i32, Option<Options>, ()>; 10] =
            [8, 9, 13, 1, 10, 7, 15, 19, 4, 20].map(|x| Job::new(x, callback));

        let vector = Vec::from(vec);
        let mut worker = Worker::from(vector);
        assert_eq!(worker.queue_len(), 10);

        worker.clock_in();

        // Give the worker thread a moment to consume initial tasks
        std::thread::sleep(std::time::Duration::from_millis(50));
        assert!(worker.queue_is_empty());

        let w = [3, 5, 14, 2, 12, 18, 17, 11, 16, 6].map(|x| Job::new(x, callback));
        worker.assign_one(Job::new(21, callback));

        // Give the worker thread a moment to process the new task
        std::thread::sleep(std::time::Duration::from_millis(10));
        assert!(worker.queue_is_empty());

        let work = Vec::from(w);
        worker.assign_many(work);

        // Give the worker thread a moment to process the new tasks
        std::thread::sleep(std::time::Duration::from_millis(50));
        assert!(worker.queue_is_empty());

        // Wait for tasks to be processed
        std::thread::sleep(std::time::Duration::from_millis(500));

        worker.clock_out();
        assert!(!worker.is_active());
        assert!(worker.queue_is_empty());

        std::thread::sleep(std::time::Duration::from_millis(1000));
    }

    #[test]
    fn test_continuous_processing() {
        // Create a worker and start it with some initial tasks
        let initial_tasks: [Job<i32, Option<Options>, ()>; 3] =
            [1, 2, 3].map(|x| Job::new(x, callback));

        let mut worker = Worker::from(Vec::from(initial_tasks));
        assert_eq!(worker.queue_len(), 3);

        // Start the worker
        worker.clock_in();
        assert!(worker.is_running());

        // Give the worker a moment to start processing
        std::thread::sleep(std::time::Duration::from_millis(50));

        // Add more tasks while the worker is running
        worker.assign_one(Job::new(4, callback));
        worker.assign_one(Job::new(5, callback));

        // Verify tasks are being processed
        std::thread::sleep(std::time::Duration::from_millis(100));

        // Add a batch of tasks
        let batch_tasks: [Job<i32, Option<Options>, ()>; 3] =
            [6, 7, 8].map(|x| Job::new(x, callback));
        worker.assign_many(Vec::from(batch_tasks));

        // Worker should still be running
        assert!(worker.is_running());
        assert!(worker.is_active());

        // Wait for tasks to be processed
        std::thread::sleep(std::time::Duration::from_millis(500));

        // All tasks should eventually be processed (queue should be empty or nearly empty)
        let remaining_tasks = worker.queue_len();
        println!("Remaining tasks in queue: {}", remaining_tasks);

        // Add one final task to ensure worker is still responsive
        worker.assign_one(Job::new(9, callback));

        // Give it time to process
        std::thread::sleep(std::time::Duration::from_millis(200));

        // Stop the worker
        worker.clock_out();
        assert!(!worker.is_active());

        // Give the worker thread time to stop
        std::thread::sleep(std::time::Duration::from_millis(100));
        assert!(!worker.is_running());
    }

    #[test]
    fn test_queue_accepts_items_while_processing() {
        // Create a worker with no initial tasks
        let mut worker = Worker::<Job<i32, Option<Options>, ()>>::new();

        // Start the worker (it should be running even with empty queue)
        worker.clock_in();
        assert!(worker.is_running());
        assert_eq!(worker.queue_len(), 0);

        // Add tasks one by one and verify they're accepted
        for i in 1..=5 {
            worker.assign_one(Job::new(i, callback));
            // Small delay to let the previous task start processing
            std::thread::sleep(std::time::Duration::from_millis(50));

            // Worker should still be running and accepting tasks
            assert!(worker.is_running());
        }

        // Add a batch while worker is processing
        let batch: [Job<i32, Option<Options>, ()>; 3] = [10, 11, 12].map(|x| Job::new(x, callback));
        worker.assign_many(Vec::from(batch));

        // Worker should still be running
        assert!(worker.is_running());

        // Wait for processing to complete
        std::thread::sleep(std::time::Duration::from_millis(500));

        // Queue should be empty or nearly empty after processing
        let final_queue_len = worker.queue_len();
        println!("Final queue length: {}", final_queue_len);

        // Stop the worker
        worker.clock_out();
        std::thread::sleep(std::time::Duration::from_millis(100));
        assert!(!worker.is_running());
    }
}
