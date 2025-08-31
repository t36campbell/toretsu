<h1 style="font-size:64px; text-align: center">堵列 <br> Toretsu</h1>

![GitHub Workflow Status (with event)](https://img.shields.io/github/actions/workflow/status/t36campbell/toretsu/workflow.yml)
![Codecov](https://img.shields.io/codecov/c/github/t36campbell/toretsu)
<br>

Toresu is a simple, multi-threaded, work-stealing task queue with a binary heap at its core, built for the Rust programming language. It can be configured as a min / max heap or even custom ordering and allows you to define the work it can do.

```rust
// Print a list of Rust keywords
fn callback<T: std::fmt::Debug>(item: T) {
    println!("Processed item {:?}", item)
}

let words: [Job<&str>; 5] =
    ["as", "break", "const", "continue", "crate"].map(|x| Job::new(x, callback));

let work = Vec::from(words);
let mut worker = Worker::from(work);

worker.clock_in();

let more_work = ["else", "extern", "false", "fn", "for"]
    .map(|x| Job::new(x, callback))
    .to_vec();

worker.assign_many(more_work);

worker.assign_one(Job::new("if", callback));
worker.assign_one(Job::new("impl", callback));
```

## Multiple Workers

Toretsu now supports running multiple workers simultaneously for improved performance on multi-core systems:

```rust
use toretsu::worker_pool::WorkerPool;

// Create a pool with 4 workers
let mut pool = WorkerPool::new(4);

// Or create from existing tasks and specify worker count
let tasks = vec![/* your tasks */];
let mut pool = WorkerPool::from(tasks, 4);

// Start all workers
pool.clock_in();

// Add more work
pool.assign_one(new_task);
pool.assign_many(more_tasks);

// Stop all workers
pool.clock_out();
```

The `WorkerPool` automatically distributes tasks across workers and can significantly improve performance for CPU-intensive workloads.

All you need to do is create a `struct` that implements the `Task` trait, which only has one method (`process`), and ensure it derives `Clone, Copy, Eq, Ord, PartialEq, PartialOrd`
```rust
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
```

## Project Goals
- Continue to add documentation to make the library as easy to use as possible
- I think I need to add redis to store a backup of the queue for failover & maintenance 
    - so it can just pick up where it left off
- Id like to limit how many constraints I have
- I want to add an `assign` method that accepts a single value or a vector
    - I tried but didnt like how the union looked - thought it was too complicated for the end-user
- The queue should mirror all methods of `std::collections::BinaryHeap`
- Id like to make this available to run via command line, like this `toretsu worker` or `toretsu workers 3`
    - I'll need to implement pub:sub messaging to add work to the queue which can be easily added with redis
