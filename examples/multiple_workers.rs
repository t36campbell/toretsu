use toretsu::task::Task;
use toretsu::worker::Worker;
use toretsu::worker_pool::WorkerPool;
use std::time::Instant;

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

fn heavy_computation(n: i32) {
    // Simulate some work
    let mut sum = 0;
    for i in 0..n * 1000 {
        sum += i;
    }
    println!("Worker processed task {}: result = {}", n, sum % 1000);
}

fn main() {
    println!("=== Toretsu Multiple Worker Demo ===\n");

    // Create a large number of tasks
    let task_count = 20;
    let jobs: Vec<Job<i32>> = (1..=task_count)
        .map(|x| Job::new(x, heavy_computation))
        .collect();

    println!("🚀 Comparing single worker vs multiple workers...\n");

    // Test with single worker
    println!("📌 Single Worker Test:");
    let start = Instant::now();
    let mut single_worker = Worker::from(jobs.clone());
    single_worker.clock_in();
    // Wait for completion
    std::thread::sleep(std::time::Duration::from_millis(1000));
    single_worker.clock_out();
    let single_duration = start.elapsed();
    println!("   Single worker completed in: {:?}\n", single_duration);

    // Test with multiple workers
    let worker_count = 4;
    println!("📌 Multiple Workers Test ({} workers):", worker_count);
    let start = Instant::now();
    let mut worker_pool = WorkerPool::from(jobs, worker_count);
    println!("   Created pool with {} workers", worker_pool.worker_count());
    println!("   Total tasks to process: {}", worker_pool.total_tasks());
    
    worker_pool.clock_in();
    println!("   All workers started...");
    
    // Wait for completion
    std::thread::sleep(std::time::Duration::from_millis(1000));
    
    worker_pool.clock_out();
    let multi_duration = start.elapsed();
    println!("   Multiple workers completed in: {:?}\n", multi_duration);

    // Show performance comparison
    if single_duration > multi_duration {
        let speedup = single_duration.as_millis() as f64 / multi_duration.as_millis() as f64;
        println!("🎉 Multiple workers were {:.2}x faster!", speedup);
    } else {
        println!("🤔 Single worker was faster (tasks might be too simple for parallelization benefits)");
    }

    println!("\n=== Demo Complete ===");
}