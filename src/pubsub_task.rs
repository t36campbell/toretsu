use serde::{Deserialize, Serialize};
use std::cmp::Ordering;

use crate::task::Task;

/// A serializable task that can be sent via Redis Pub/Sub
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PubSubTask {
    pub id: String,
    pub payload: String,
    pub priority: i32, // Lower numbers = higher priority (for min-heap behavior)
}

impl PubSubTask {
    pub fn new(id: String, payload: String, priority: i32) -> Self {
        Self {
            id,
            payload,
            priority,
        }
    }

    /// Create a PubSubTask from JSON string
    pub fn from_json(json: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(json)
    }

    /// Convert PubSubTask to JSON string
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string(self)
    }
}

impl Task for PubSubTask {
    fn process(&mut self) {
        println!("Processing task {}: {}", self.id, self.payload);
    }
}

impl Ord for PubSubTask {
    fn cmp(&self, other: &Self) -> Ordering {
        // Reverse order for min-heap behavior (lower priority number = higher priority)
        other.priority.cmp(&self.priority)
    }
}

impl PartialOrd for PubSubTask {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pubsub_task_creation() {
        let task = PubSubTask::new("test-1".to_string(), "Hello World".to_string(), 1);
        assert_eq!(task.id, "test-1");
        assert_eq!(task.payload, "Hello World");
        assert_eq!(task.priority, 1);
    }

    #[test]
    fn test_pubsub_task_serialization() {
        let task = PubSubTask::new("test-1".to_string(), "Hello World".to_string(), 1);
        let json = task.to_json().unwrap();
        let deserialized = PubSubTask::from_json(&json).unwrap();
        assert_eq!(task, deserialized);
    }

    #[test]
    fn test_pubsub_task_priority_ordering() {
        let task1 = PubSubTask::new("test-1".to_string(), "High Priority".to_string(), 1);
        let task2 = PubSubTask::new("test-2".to_string(), "Low Priority".to_string(), 10);

        // task1 should be greater than task2 (higher priority)
        assert!(task1 > task2);
    }

    #[test]
    fn test_task_process() {
        let mut task = PubSubTask::new("test-1".to_string(), "Test payload".to_string(), 1);
        // This should not panic
        task.process();
    }
}
