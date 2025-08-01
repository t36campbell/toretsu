use names::{Generator, Name};
use redis::{ControlFlow, Msg};
use std::sync::{Arc, Mutex};
use uuid::Uuid;

use crate::client::Client;
use crate::pubsub_task::PubSubTask;
use crate::queue::Queue;
use crate::task::Task;

pub struct Worker<T> {
    pub id: Uuid,
    pub channel: String,
    pub queue: Queue<T>,
    pub active: bool,
}

impl<T> Default for Worker<T>
where
    T: Task + Ord + std::marker::Send + 'static,
{
    fn default() -> Self {
        Self::new()
    }
}

impl<T> Worker<T>
where
    T: Task + Ord + std::marker::Send + 'static,
{
    fn generate_name() -> String {
        let mut generator = Generator::with_naming(Name::Numbered);
        match generator.next() {
            Some(name) => name,
            None => String::from("Default Worker"),
        }
    }

    pub fn new() -> Self {
        Self {
            id: Uuid::new_v4(),
            queue: Queue::default(),
            channel: Self::generate_name(),
            active: true,
        }
    }

    pub fn from(vec: Vec<T>) -> Self {
        Self {
            id: Uuid::new_v4(),
            queue: Queue::from(vec),
            channel: Self::generate_name(),
            active: true,
        }
    }

    pub fn clock_in(&mut self) {
        self.active = true;

        for mut item in self.queue.drain_sorted() {
            rayon::spawn(move || item.process());
        }
    }

    pub fn clock_out(&mut self) {
        self.active = false
    }

    pub fn assign_one(&mut self, task: T) {
        self.queue.push(task);
    }

    pub fn assign_many<I: IntoIterator<Item = T>>(&mut self, iter: I) {
        self.queue.extend(iter);
    }

    /// Start listening to Redis pub/sub for PubSubTask messages
    /// This method works specifically with PubSubTask and spawns a background thread
    pub fn listen_for_pubsub_tasks(&mut self, channels: Vec<String>)
    where
        T: From<PubSubTask> + 'static,
    {
        if !self.active {
            return;
        }

        let worker_channel = self.channel.clone();
        println!("Worker {worker_channel} starting to listen on channels: {channels:?}");

        // Create a new Redis client for pub/sub listening
        match Client::try_new() {
            Ok(mut client) => {
                client.listen(channels, move |msg: Msg| -> ControlFlow<()> {
                    let payload: String = match msg.get_payload() {
                        Ok(p) => p,
                        Err(_) => return ControlFlow::Continue,
                    };

                    match PubSubTask::from_json(&payload) {
                        Ok(task) => {
                            println!(
                                "Worker {} received task: {} with payload: {}",
                                worker_channel, task.id, task.payload
                            );
                            // Here we would add the task to the queue if we had shared access
                            // For now, we just process it directly
                            let mut task_clone = task.clone();
                            rayon::spawn(move || {
                                task_clone.process();
                            });
                        }
                        Err(e) => {
                            eprintln!("Failed to parse task from Redis message: {e}");
                        }
                    }

                    ControlFlow::Continue
                });
            }
            Err(e) => {
                eprintln!("Worker {worker_channel} failed to connect to Redis: {e}");
            }
        }
    }
}

/// A specialized worker that can handle PubSubTasks with Redis integration
pub struct PubSubWorker {
    pub id: Uuid,
    pub channel: String,
    pub queue: Arc<Mutex<Queue<PubSubTask>>>,
    pub active: bool,
}

impl Default for PubSubWorker {
    fn default() -> Self {
        Self::new()
    }
}

impl PubSubWorker {
    fn generate_name() -> String {
        let mut generator = Generator::with_naming(Name::Numbered);
        match generator.next() {
            Some(name) => name,
            None => String::from("Default PubSub Worker"),
        }
    }

    pub fn new() -> Self {
        Self {
            id: Uuid::new_v4(),
            channel: Self::generate_name(),
            queue: Arc::new(Mutex::new(Queue::default())),
            active: true,
        }
    }

    pub fn from(tasks: Vec<PubSubTask>) -> Self {
        Self {
            id: Uuid::new_v4(),
            channel: Self::generate_name(),
            queue: Arc::new(Mutex::new(Queue::from(tasks))),
            active: true,
        }
    }

    pub fn clock_in(&mut self) {
        self.active = true;
        self.process_queue();
    }

    pub fn clock_out(&mut self) {
        self.active = false;
    }

    pub fn process_queue(&self) {
        let queue_clone = Arc::clone(&self.queue);
        rayon::spawn(move || {
            loop {
                let task = {
                    let mut queue = queue_clone.lock().unwrap();
                    queue.pop()
                };

                match task {
                    Some(mut task) => {
                        task.process();
                    }
                    None => {
                        // The queue is empty, so we can stop this processing thread.
                        break;
                    }
                }
            }
        });
    }

    pub fn assign_one(&self, task: PubSubTask) {
        let mut queue = self.queue.lock().unwrap();
        queue.push(task);
    }

    pub fn assign_many(&self, tasks: Vec<PubSubTask>) {
        let mut queue = self.queue.lock().unwrap();
        queue.extend(tasks);
    }

    pub fn queue_len(&self) -> usize {
        let queue = self.queue.lock().unwrap();
        queue.len()
    }

    pub fn is_queue_empty(&self) -> bool {
        let queue = self.queue.lock().unwrap();
        queue.is_empty()
    }

    /// Start listening to Redis pub/sub channels and automatically add tasks to queue
    pub fn start_pubsub_listener(&self, channels: Vec<String>) {
        if !self.active {
            return;
        }

        let worker_channel = self.channel.clone();
        let queue_clone = Arc::clone(&self.queue);

        println!("PubSubWorker {worker_channel} starting to listen on channels: {channels:?}");

        rayon::spawn(move || {
            match Client::try_new() {
                Ok(mut client) => {
                    client.listen(channels, move |msg: Msg| -> ControlFlow<()> {
                        let payload: String = match msg.get_payload() {
                            Ok(p) => p,
                            Err(_) => return ControlFlow::Continue,
                        };

                        match PubSubTask::from_json(&payload) {
                            Ok(task) => {
                                println!(
                                    "PubSubWorker {} received task: {} with payload: {}",
                                    worker_channel, task.id, task.payload
                                );

                                // Add task to the shared queue
                                let mut queue = queue_clone.lock().unwrap();
                                queue.push(task);
                            }
                            Err(e) => {
                                eprintln!("Failed to parse task from Redis message: {e}");
                            }
                        }

                        ControlFlow::Continue
                    });
                }
                Err(e) => {
                    eprintln!("PubSubWorker {worker_channel} failed to connect to Redis: {e}");
                }
            }
        });
    }
}
