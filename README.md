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
- ✅ I think I need to add redis to store a backup of the queue for failover & maintenance 
    - so it can just pick up where it left off
- Id like to limit how many constraints I have
- I want to add an `assign` method that accepts a single value or a vector
    - I tried but didnt like how the union looked - thought it was too complicated for the end-user
- The queue should mirror all methods of `std::collections::BinaryHeap`
- Id like to make this available to run via command line, like this `toretsu worker` or `toretsu workers 3`
    - I'll need to implement pub:sub messaging to add work to the queue which can be easily added with redis

## Redis Backup Feature

Toretsu now supports backing up queue data to Redis for failover and recovery scenarios. This allows workers to restore their queue state after failures or restarts.

### Basic Usage

```rust
use serde::{Deserialize, Serialize};
use toretsu::{client::Client, queue::Queue, task::Task, worker::Worker};

#[derive(Clone, Copy, Eq, Ord, PartialEq, PartialOrd, Serialize, Deserialize)]
struct Job {
    id: u32,
    priority: u32,
}

impl Task for Job {
    fn process(&mut self) {
        println!("Processing job {} with priority {}", self.id, self.priority);
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create a worker with some jobs
    let jobs = vec![
        Job { id: 1, priority: 10 },
        Job { id: 2, priority: 5 },
        Job { id: 3, priority: 15 },
    ];
    let mut worker = Worker::from(jobs);
    let queue_id = worker.queue.id;

    // Connect to Redis
    let mut client = Client::new();

    // Backup the queue
    worker.backup_queue(&mut client)?;

    // Simulate failure and recovery
    drop(worker);

    // Restore worker from backup
    let mut restored_worker = Worker::new_with_restore(queue_id, &mut client);
    
    // Process all jobs
    restored_worker.clock_in();

    Ok(())
}
```

### API Methods

#### Queue Methods
- `backup_to_redis(&self, client: &mut Client) -> RedisResult<()>` - Backup queue data to Redis
- `restore_from_redis(id: Uuid, client: &mut Client) -> RedisResult<Self>` - Restore queue from Redis
- `delete_backup_from_redis(&self, client: &mut Client) -> RedisResult<()>` - Delete backup from Redis

#### Worker Methods
- `backup_queue(&self, client: &mut Client) -> RedisResult<()>` - Backup the worker's queue
- `restore_queue(&mut self, client: &mut Client) -> RedisResult<()>` - Restore the worker's queue
- `new_with_restore(queue_id: Uuid, client: &mut Client) -> Self` - Create worker and restore from backup
- `delete_queue_backup(&self, client: &mut Client) -> RedisResult<()>` - Delete queue backup

### Requirements

For backup functionality to work, your task types must implement:
- `Serialize` and `Deserialize` from serde
- `Clone` (for backup operations)
- `Ord` (required by the underlying BinaryHeap)

### Example

Run the backup example to see the functionality in action:

```bash
cargo run --example backup_example
```
