#[cfg(test)]
mod tests {
    use toretsu::task::Task;
    use toretsu::worker_pool::WorkerPool;

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
        println!("Processed item {:?}", item)
    }

    #[test]
    fn test_worker_pool_new() {
        let pool: WorkerPool<Job<i32>> = WorkerPool::new(3);
        assert_eq!(pool.worker_count(), 3);
        assert_eq!(pool.total_tasks(), 0);
    }

    #[test]
    fn test_worker_pool_default() {
        let pool: WorkerPool<Job<i32>> = WorkerPool::default();
        assert!(pool.worker_count() > 0); // Should use num_cpus
    }

    #[test]
    fn test_worker_pool_from() {
        let jobs: Vec<Job<i32>> = vec![1, 2, 3, 4, 5]
            .into_iter()
            .map(|x| Job::new(x, callback))
            .collect();
        
        let pool = WorkerPool::from(jobs, 2);
        assert_eq!(pool.worker_count(), 2);
        assert_eq!(pool.total_tasks(), 5);
    }

    #[test]
    fn test_assign_tasks() {
        let mut pool: WorkerPool<Job<i32>> = WorkerPool::new(2);
        
        pool.assign_one(Job::new(1, callback));
        assert_eq!(pool.total_tasks(), 1);
        
        let more_jobs: Vec<Job<i32>> = vec![2, 3, 4]
            .into_iter()
            .map(|x| Job::new(x, callback))
            .collect();
        pool.assign_many(more_jobs);
        assert_eq!(pool.total_tasks(), 4);
    }

    #[test]
    fn test_worker_pool_clock_operations() {
        let jobs: Vec<Job<i32>> = vec![1, 2, 3, 4, 5, 6]
            .into_iter()
            .map(|x| Job::new(x, callback))
            .collect();
        
        let mut pool = WorkerPool::from(jobs, 3);
        assert_eq!(pool.worker_count(), 3);
        assert_eq!(pool.total_tasks(), 6);
        
        // Initially workers should not be active
        assert!(!pool.all_active());
        
        pool.clock_in();
        // After clock_in, workers should be active and tasks should be distributed/processed
        assert!(pool.all_active());
        
        // Give a moment for tasks to be processed (they run async)
        std::thread::sleep(std::time::Duration::from_millis(100));
        
        pool.clock_out();
        assert!(!pool.any_active());
    }

    #[test]
    fn test_multiple_workers_processing() {
        use std::sync::atomic::{AtomicUsize, Ordering};
        
        static COUNTER: AtomicUsize = AtomicUsize::new(0);
        
        fn increment_callback(_: i32) {
            COUNTER.fetch_add(1, Ordering::SeqCst);
        }

        let jobs: Vec<Job<i32>> = (1..=20)
            .map(|x| Job::new(x, increment_callback))
            .collect();
        
        let mut pool = WorkerPool::from(jobs, 4);
        pool.clock_in();
        
        // Wait for all tasks to complete
        std::thread::sleep(std::time::Duration::from_millis(500));
        
        pool.clock_out();
        
        // All 20 tasks should have been processed
        assert_eq!(COUNTER.load(Ordering::SeqCst), 20);
    }
}