use names::{Generator, Name};
use uuid::Uuid;

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
        generator.next().unwrap()
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

        let queue = match queue {
            Some(queue) => queue,
            None => Queue::new(),
        };

        Self {
            id,
            channel,
            active: true,
            queue,
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
        let worker = std::mem::take(self);

        rayon::spawn(move || {
            for mut item in worker.queue {
                rayon::spawn(move || item.process());
            }
        });
    }

    pub fn clock_out(&mut self) {
        self.active = false
    }

    pub fn assign_one(&mut self, task: T) {
        self.queue.push(task);
        self.clock_in();
    }

    pub fn assign_many(&mut self, vec: Vec<T>) {
        for task in vec {
            self.queue.push(task);
        }
        self.clock_in();
    }
}
