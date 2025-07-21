use names::{Generator, Name};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;
use uuid::Uuid;

use crate::queue::Queue;
use crate::task::Task;

enum WorkerState<T> {
    Idle(Queue<T>),
    Running {
        shared_queue: Arc<Mutex<Queue<T>>>,
        shared_active: Arc<Mutex<bool>>,
    },
}

pub struct Worker<T> {
    pub id: Uuid,
    pub channel: String,
    state: WorkerState<T>,
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
            channel,
            state: WorkerState::Idle(queue),
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
        if matches!(self.state, WorkerState::Running { .. }) {
            return;
        }

        let queue = match std::mem::replace(&mut self.state, WorkerState::Idle(Queue::new())) {
            WorkerState::Idle(queue) => queue,
            WorkerState::Running { .. } => unreachable!(),
        };

        let shared_queue = Arc::new(Mutex::new(queue));
        let shared_active = Arc::new(Mutex::new(true));

        self.state = WorkerState::Running {
            shared_queue: Arc::clone(&shared_queue),
            shared_active: Arc::clone(&shared_active),
        };

        thread::spawn(move || loop {
            {
                let is_active = shared_active.lock().unwrap();
                if !*is_active {
                    break;
                }
            };

            let task = {
                let mut guard = shared_queue.lock().unwrap();
                guard.pop()
            };

            match task {
                Some(mut t) => {
                    rayon::spawn(move || t.process());
                }
                None => {
                    thread::sleep(Duration::from_millis(10));
                }
            }
        });
    }

    pub fn clock_out(&mut self) {
        let (shared_queue, shared_active) = match &self.state {
            WorkerState::Idle(_) => return,
            WorkerState::Running {
                shared_queue,
                shared_active,
            } => (Arc::clone(shared_queue), Arc::clone(shared_active)),
        };

        {
            let mut active = shared_active.lock().unwrap();
            *active = false;
        }

        thread::sleep(Duration::from_millis(50));
        let recovered_queue = match Arc::try_unwrap(shared_queue) {
            Ok(mutex) => mutex.into_inner().unwrap(),
            Err(shared_queue) => {
                if let Ok(mut queue_guard) = shared_queue.lock() {
                    let remaining_tasks: Vec<_> = queue_guard.drain().collect();
                    Queue::from(remaining_tasks)
                } else {
                    Queue::new()
                }
            }
        };

        self.state = WorkerState::Idle(recovered_queue);
    }

    pub fn is_running(&self) -> bool {
        matches!(self.state, WorkerState::Running { .. })
    }

    pub fn queue_len(&self) -> usize {
        match &self.state {
            WorkerState::Idle(queue) => queue.len(),
            WorkerState::Running { shared_queue, .. } => {
                let queue = shared_queue.lock().unwrap();
                queue.len()
            }
        }
    }

    pub fn queue_is_empty(&self) -> bool {
        match &self.state {
            WorkerState::Idle(queue) => queue.is_empty(),
            WorkerState::Running { shared_queue, .. } => {
                let queue = shared_queue.lock().unwrap();
                queue.is_empty()
            }
        }
    }

    pub fn is_active(&self) -> bool {
        match &self.state {
            WorkerState::Idle(_) => false,
            WorkerState::Running { shared_active, .. } => {
                let active = shared_active.lock().unwrap();
                *active
            }
        }
    }

    pub fn assign_one(&mut self, task: T) {
        match &mut self.state {
            WorkerState::Idle(queue) => queue.push(task),
            WorkerState::Running { shared_queue, .. } => {
                let mut queue = shared_queue.lock().unwrap();
                queue.push(task);
            }
        }
    }

    pub fn assign_many(&mut self, vec: Vec<T>) {
        match &mut self.state {
            WorkerState::Idle(queue) => {
                queue.extend(vec);
            }
            WorkerState::Running { shared_queue, .. } => {
                let mut queue = shared_queue.lock().unwrap();
                queue.extend(vec);
            }
        }
    }
}
