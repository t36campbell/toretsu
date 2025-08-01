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

## Redis Pub/Sub Support

Toretsu now supports adding tasks via Redis Pub/Sub! You can use the `PubSubWorker` to automatically process tasks published to Redis channels.

```rust
use toretsu::{client::Client, pubsub_task::PubSubTask, worker::PubSubWorker};

// Create a worker that listens to Redis channels
let mut worker = PubSubWorker::new();
worker.clock_in();

// Start listening to Redis channels
let channels = vec!["tasks".to_string(), "urgent_tasks".to_string()];
worker.start_pubsub_listener(channels);

// Publish tasks via Redis
let mut client = Client::new();
let task = PubSubTask::new("task-1".to_string(), "Process data".to_string(), 1);
let task_json = task.to_json().unwrap();
client.publish("tasks", task_json).unwrap();

// The worker will automatically receive and process the task!
```

You can also publish tasks from the command line:
```bash
redis-cli PUBLISH tasks '{"id":"task-1","payload":"Hello World","priority":1}'
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
- I think I need to add redis to store a backup of the queue for failover & maintenance 
    - so it can just pick up where it left off
- Id like to limit how many constraints I have
- I want to add an `assign` method that accepts a single value or a vector
    - I tried but didnt like how the union looked - thought it was too complicated for the end-user
- The queue should mirror all methods of `std::collections::BinaryHeap`
- Id like to make this available to run via command line, like this `toretsu worker` or `toretsu workers 3`
    - ✅ **IMPLEMENTED**: Pub/sub messaging to add work to the queue is now available with Redis
