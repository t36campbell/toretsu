use names::{Generator, Name};
use redis::RedisResult;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::client::Client;
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

    pub fn init(id: Option<Uuid>, channel: Option<String>, queue: Option<Queue<T>>) -> Self {
        let id = match id {
            Some(id) => id,
            None => Uuid::new_v4(),
        };

        let channel = match channel {
            Some(channel) => channel,
            None => Self::generate_name(),
        };

        let queue = queue.unwrap_or_default();

        Self {
            id,
            queue,
            channel,
            active: true,
        }
    }

    pub fn new() -> Self {
        Self::init(None, None, None)
    }

    pub fn from(vec: Vec<T>) -> Self {
        let queue = Queue::from(vec);

        Self::init(None, None, Some(queue))
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

    /// Persist the worker's queue to Redis
    /// Returns Ok(()) if successful, Err if backup failed
    pub fn backup_queue(&self, client: &mut Client) -> RedisResult<()>
    where
        T: Serialize + for<'de> Deserialize<'de> + Clone,
    {
        self.queue.persist(client)
    }

    /// Restore the worker's queue from Redis using the queue's current UUID
    /// Returns Ok(()) if successful, Err if restore failed
    pub fn restore_queue(&mut self, client: &mut Client) -> RedisResult<()>
    where
        T: for<'de> Deserialize<'de> + Ord + Serialize + Clone,
    {
        match Queue::restore_from_redis(self.queue.id, client) {
            Ok(restored_queue) => {
                self.queue = restored_queue;
                Ok(())
            }
            Err(e) => Err(e),
        }
    }

    /// Create a new worker with a specific queue ID and attempt to restore from Redis
    /// If restore fails, creates a new empty queue with the given ID
    pub fn new_with_restore(queue_id: Uuid, client: &mut Client) -> Self
    where
        T: for<'de> Deserialize<'de> + Ord + Serialize + Clone,
    {
        let queue = Queue::restore_from_redis(queue_id, client)
            .unwrap_or_else(|_| Queue::init(queue_id, Vec::new()));

        Self::init(None, None, Some(queue))
    }

    /// Delete the queue backup from Redis
    pub fn delete_queue_backup(&self, client: &mut Client) -> RedisResult<()>
    where
        T: Serialize + for<'de> Deserialize<'de> + Clone,
    {
        self.queue.delete_from_redis(client)
    }
}
