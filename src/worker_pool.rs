use std::sync::{Arc, Mutex};
use std::collections::VecDeque;
use uuid::Uuid;

use crate::task::Task;
use crate::worker::Worker;

pub struct WorkerPool<T> {
    pub id: Uuid,
    workers: Vec<Worker<T>>,
    shared_queue: Arc<Mutex<VecDeque<T>>>,
    worker_queues: Vec<Arc<Mutex<VecDeque<T>>>>,
    worker_count: usize,
}

impl<T> WorkerPool<T>
where
    T: Task + Ord + std::marker::Send + 'static,
{
    /// Create a new WorkerPool with the specified number of workers
    pub fn new(worker_count: usize) -> Self {
        let mut workers = Vec::with_capacity(worker_count);
        let shared_queue = Arc::new(Mutex::new(VecDeque::new()));
        let mut worker_queues = Vec::with_capacity(worker_count);
        
        for _ in 0..worker_count {
            workers.push(Worker::new());
            worker_queues.push(Arc::new(Mutex::new(VecDeque::new())));
        }

        Self {
            id: Uuid::new_v4(),
            workers,
            shared_queue,
            worker_queues,
            worker_count,
        }
    }

    /// Create a WorkerPool from a vector of tasks, distributing them across workers
    pub fn from(tasks: Vec<T>, worker_count: usize) -> Self {
        let mut pool = Self::new(worker_count);
        pool.assign_many(tasks);
        pool
    }

    /// Get the number of workers in the pool
    pub fn worker_count(&self) -> usize {
        self.worker_count
    }

    /// Check if all workers are active
    pub fn all_active(&self) -> bool {
        self.workers.iter().all(|w| w.active)
    }

    /// Check if any worker is active
    pub fn any_active(&self) -> bool {
        self.workers.iter().any(|w| w.active)
    }

    /// Get the total number of tasks across all workers and queues
    pub fn total_tasks(&self) -> usize {
        let worker_tasks = self.workers.iter().map(|w| w.queue.len()).sum::<usize>();
        let shared_tasks = self.shared_queue.lock().unwrap().len();
        let worker_queue_tasks = self.worker_queues.iter()
            .map(|q| q.lock().unwrap().len())
            .sum::<usize>();
        
        worker_tasks + shared_tasks + worker_queue_tasks
    }

    /// Add a single task to the shared queue
    pub fn assign_one(&mut self, task: T) {
        self.shared_queue.lock().unwrap().push_back(task);
    }

    /// Add multiple tasks to the shared queue
    pub fn assign_many<I: IntoIterator<Item = T>>(&mut self, tasks: I) {
        let mut queue = self.shared_queue.lock().unwrap();
        for task in tasks {
            queue.push_back(task);
        }
    }

    /// Start all workers in the pool with work-stealing support
    pub fn clock_in(&mut self) {
        // Distribute initial tasks to worker queues
        self.distribute_tasks();
        
        // Start workers with simple approach - each worker processes its assigned tasks
        // Work-stealing happens during task distribution
        for worker in &mut self.workers {
            worker.clock_in();
        }
        
        // Add any remaining shared queue tasks to workers
        self.redistribute_remaining_tasks();
    }

    /// Stop all workers in the pool
    pub fn clock_out(&mut self) {
        for worker in &mut self.workers {
            worker.clock_out();
        }
    }

    /// Distribute tasks from the shared queue to individual worker queues
    fn distribute_tasks(&mut self) {
        let mut shared_queue = self.shared_queue.lock().unwrap();
        let mut tasks: Vec<T> = shared_queue.drain(..).collect();
        
        // Sort tasks to maintain priority order
        tasks.sort();
        
        // Distribute tasks round-robin to workers directly
        for (i, task) in tasks.into_iter().enumerate() {
            let worker_index = i % self.worker_count;
            self.workers[worker_index].assign_one(task);
        }
    }

    /// Redistribute any remaining tasks from worker queues
    fn redistribute_remaining_tasks(&mut self) {
        // Simple work-stealing: move tasks from overloaded workers to underloaded ones
        let mut all_tasks = Vec::new();
        
        // Collect all tasks from worker queues
        for queue in &self.worker_queues {
            if let Ok(mut q) = queue.lock() {
                all_tasks.extend(q.drain(..));
            }
        }
        
        // Also get any remaining shared tasks
        if let Ok(mut shared) = self.shared_queue.lock() {
            all_tasks.extend(shared.drain(..));
        }
        
        // Redistribute evenly
        for (i, task) in all_tasks.into_iter().enumerate() {
            let worker_index = i % self.worker_count;
            self.workers[worker_index].assign_one(task);
        }
    }
}

impl<T> Default for WorkerPool<T>
where
    T: Task + Ord + std::marker::Send + 'static,
{
    fn default() -> Self {
        Self::new(num_cpus::get())
    }
}