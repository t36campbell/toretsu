use std::sync::{Arc, Mutex};
use std::collections::VecDeque;
use uuid::Uuid;

use crate::task::Task;
use crate::worker::Worker;

pub struct WorkerPool<T> {
    pub id: Uuid,
    workers: Vec<Worker<T>>,
    shared_queue: Arc<Mutex<VecDeque<T>>>,
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
        
        for _ in 0..worker_count {
            workers.push(Worker::new());
        }

        Self {
            id: Uuid::new_v4(),
            workers,
            shared_queue,
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

    /// Get the total number of tasks across all workers
    pub fn total_tasks(&self) -> usize {
        self.workers.iter().map(|w| w.queue.len()).sum::<usize>() 
            + self.shared_queue.lock().unwrap().len()
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

    /// Start all workers in the pool
    pub fn clock_in(&mut self) {
        // Distribute tasks from shared queue to workers
        self.distribute_tasks();
        
        // Start all workers
        for worker in &mut self.workers {
            worker.clock_in();
        }
    }

    /// Stop all workers in the pool
    pub fn clock_out(&mut self) {
        for worker in &mut self.workers {
            worker.clock_out();
        }
    }

    /// Distribute tasks from the shared queue to individual workers
    fn distribute_tasks(&mut self) {
        let mut shared_queue = self.shared_queue.lock().unwrap();
        let mut tasks: Vec<T> = shared_queue.drain(..).collect();
        
        // Sort tasks to maintain priority order
        tasks.sort();
        
        // Distribute tasks round-robin to workers
        for (i, task) in tasks.into_iter().enumerate() {
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