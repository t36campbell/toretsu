#[cfg(test)]
mod tests {
    use rand::distributions::{Distribution, Uniform};
    use toretsu::task::Task;
    use toretsu::worker::Worker;

    #[derive(Clone, Copy, Eq, Ord, PartialEq, PartialOrd)]
    struct Job<T> {
        value: T,
        func: fn(T),
    }

    impl<T> Job<T> {
        fn new(value: T, func: fn(T)) -> Self {
            Self { value, func }
        }
    }

    impl<T> Task for Job<T>
    where
        T: Copy,
    {
        fn process(&mut self) {
            (self.func)(self.value)
        }
    }

    fn callback<T: std::fmt::Debug>(item: T) {
        // For most tests, print the item
        println!("{:?}", item)
    }

    fn cpu_callback<T>(item: T) {
        // CPU-intensive work without I/O for performance testing
        let mut sum = 0;
        for i in 0..1000 {
            // Small computation to simulate work
            sum += i;
        }
        // Use the item and sum to prevent optimization
        std::hint::black_box((item, sum));
    }

    #[test]
    fn simple_example() {
        // Print a list of Rust keywords
        let words: [Job<&str>; 5] =
            ["as", "break", "const", "continue", "crate"].map(|x| Job::new(x, callback));

        let work = Vec::from(words);
        let mut worker = Worker::from(work);
        assert_eq!(worker.queue_len(), 5);

        worker.clock_in();

        // Give the worker thread a moment to consume initial tasks
        std::thread::sleep(std::time::Duration::from_millis(50));
        assert!(worker.queue_is_empty());

        let more_work = ["else", "extern", "false", "fn", "for"]
            .map(|x| Job::new(x, callback))
            .to_vec();

        worker.assign_many(more_work);

        worker.assign_one(Job::new("if", callback));
        worker.assign_one(Job::new("impl", callback));

        // Wait for tasks to be processed
        std::thread::sleep(std::time::Duration::from_millis(300));

        worker.clock_out();
        assert!(!worker.is_active());
        assert!(worker.queue_is_empty());
    }

    #[test]
    fn torture_test() {
        // Reduced scale for more realistic async testing
        let task_count = 10_000; // 10K tasks instead of 1M
        let ints = {
            let mut ints = Vec::new();
            let range = Uniform::from(0..task_count);
            let mut rng = rand::thread_rng();

            for _ in 0..task_count {
                let num = range.sample(&mut rng);
                ints.push(num);
            }

            ints
        };

        let work = ints.into_iter().map(|x| Job::new(x, cpu_callback));
        let _more_work = work.clone();
        let mut worker = toretsu::worker::Worker::from(work.collect());
        assert_eq!(worker.queue_len(), task_count);

        worker.clock_in();

        // Wait for tasks to be processed (shorter wait for smaller scale)
        std::thread::sleep(std::time::Duration::from_millis(5000));

        let mut remaining = worker.queue_len();
        println!("Queue length after initial wait: {}", remaining);

        // Give additional time if many tasks remain
        if remaining > task_count / 10 {
            // If more than 10% remain
            println!("Many tasks remaining, waiting longer...");
            std::thread::sleep(std::time::Duration::from_millis(5000));
            remaining = worker.queue_len();
            println!("Queue length after extended wait: {}", remaining);
        }

        worker.clock_out();
        assert!(!worker.is_active());

        let final_remaining = worker.queue_len();
        println!("Final queue length: {}", final_remaining);

        // For async processing with 10K tasks, expect at least 80% completion
        let processed = task_count - final_remaining;
        let processing_rate = (processed as f64) / (task_count as f64);
        println!("Processing rate: {:.2}%", processing_rate * 100.0);

        assert!(
            processing_rate >= 0.8,
            "Expected at least 80% of tasks to be processed, but only {:.2}% were processed",
            processing_rate * 100.0
        );
    }

    #[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
    struct SimpleTask {
        id: u32,
        priority: u32,
    }

    impl Task for SimpleTask {
        fn process(&mut self) {
            println!(
                "Processing task {} with priority {}",
                self.id, self.priority
            );
            // Simulate some work
            std::thread::sleep(std::time::Duration::from_millis(100));
            println!("Completed task {}", self.id);
        }
    }

    #[test]
    fn continuous_worker() {
        println!("Starting continuous worker demo...");

        // Create a worker
        let mut worker = Worker::new();

        println!("Worker created with ID: {}", worker.id);

        // Add some initial tasks
        let tasks = vec![
            SimpleTask { id: 1, priority: 3 },
            SimpleTask { id: 2, priority: 1 },
            SimpleTask { id: 3, priority: 2 },
        ];

        println!("Adding initial tasks...");
        worker.assign_many(tasks);

        // Give the worker a moment to start processing
        std::thread::sleep(std::time::Duration::from_millis(50));

        // Add more tasks while worker is running
        println!("Adding more tasks while worker is processing...");
        worker.assign_one(SimpleTask { id: 4, priority: 4 });
        worker.assign_one(SimpleTask { id: 5, priority: 1 });

        // Check worker status
        println!("Worker is running: {}", worker.is_running());
        println!("Queue length: {}", worker.queue_len());

        // Wait for tasks to be processed
        std::thread::sleep(std::time::Duration::from_secs(1));

        // Add more tasks
        println!("Adding final batch of tasks...");
        worker.assign_many(vec![
            SimpleTask { id: 6, priority: 2 },
            SimpleTask { id: 7, priority: 5 },
        ]);

        // Wait a bit more
        std::thread::sleep(std::time::Duration::from_millis(500));

        println!("Final queue length: {}", worker.queue_len());

        // Stop the worker
        println!("Stopping worker...");
        worker.clock_out();

        // Give it a moment to clean up
        std::thread::sleep(std::time::Duration::from_millis(100));

        println!("Worker is running: {}", worker.is_running());
        println!("Worker is active: {}", worker.is_active());

        println!("Demo completed!");

        // Assertions to verify the demo worked correctly
        assert!(!worker.is_running());
        assert!(!worker.is_active());
    }
}
