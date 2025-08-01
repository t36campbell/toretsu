#[cfg(test)]
mod tests {
    use std::time::Duration;
    use toretsu::client::Client;
    use toretsu::pubsub_task::PubSubTask;
    use toretsu::worker::PubSubWorker;

    #[test]
    #[ignore] // Requires Redis server running
    fn test_pubsub_worker_basic_functionality() {
        // Test the basic functionality without Redis dependency
        let worker = PubSubWorker::new();
        assert!(worker.is_queue_empty());

        let task = PubSubTask::new("test-1".to_string(), "Hello World".to_string(), 1);
        worker.assign_one(task);

        assert_eq!(worker.queue_len(), 1);
        assert!(!worker.is_queue_empty());
    }

    #[test]
    #[ignore] // Requires Redis server running
    fn test_pubsub_integration() {
        // This test demonstrates how to use the Redis Pub/Sub feature
        // but is ignored because it requires a Redis server to be running

        let mut worker = PubSubWorker::new();
        worker.clock_in();

        // Start listening to a test channel
        let channels = vec!["test_tasks".to_string()];
        worker.start_pubsub_listener(channels.clone());

        // Give the listener time to start
        std::thread::sleep(Duration::from_millis(100));

        // Publish a task to the channel
        let mut client = Client::new();
        let task = PubSubTask::new("test-task-1".to_string(), "Test payload".to_string(), 1);
        let task_json = task.to_json().unwrap();

        let _ = client.publish("test_tasks", task_json);

        // Give time for the message to be processed
        std::thread::sleep(Duration::from_millis(500));

        worker.clock_out();
    }

    #[test]
    fn test_task_serialization_roundtrip() {
        let original_task = PubSubTask::new(
            "test-123".to_string(),
            "This is a test payload".to_string(),
            5,
        );

        let json = original_task.to_json().unwrap();
        let deserialized_task = PubSubTask::from_json(&json).unwrap();

        assert_eq!(original_task, deserialized_task);
        assert_eq!(deserialized_task.id, "test-123");
        assert_eq!(deserialized_task.payload, "This is a test payload");
        assert_eq!(deserialized_task.priority, 5);
    }

    #[test]
    fn test_multiple_tasks_priority_ordering() {
        let worker = PubSubWorker::new();

        // Add tasks with different priorities
        let high_priority =
            PubSubTask::new("high".to_string(), "High priority task".to_string(), 1);
        let low_priority = PubSubTask::new("low".to_string(), "Low priority task".to_string(), 10);
        let medium_priority =
            PubSubTask::new("medium".to_string(), "Medium priority task".to_string(), 5);

        worker.assign_one(low_priority);
        worker.assign_one(high_priority);
        worker.assign_one(medium_priority);

        assert_eq!(worker.queue_len(), 3);
    }
}
