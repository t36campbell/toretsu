use std::thread;
use std::time::Duration;
use toretsu::pubsub_task::PubSubTask;
use toretsu::worker::PubSubWorker;
use uuid::Uuid;

/// Helper function to check for a Redis connection.
fn can_connect_to_redis() -> bool {
    redis::Client::open("redis://127.0.0.1/").is_ok()
}

#[test]
#[ignore] // This is an integration test and requires a running Redis server.
fn test_pubsub_listener_receives_task() {
    if !can_connect_to_redis() {
        println!("Skipping test: Redis server not available at redis://127.0.0.1/");
        return;
    }

    // 1. Setup
    let worker = PubSubWorker::new();
    let channel_name = format!("test-channel-{}", Uuid::new_v4());
    let channels = vec![channel_name.clone()];

    // 2. Start the listener in the background
    worker.start_pubsub_listener(channels);

    // Give the listener a moment to subscribe
    thread::sleep(Duration::from_millis(50));

    // 3. Publish a task to the channel from a separate client
    let redis_client =
        redis::Client::open("redis://127.0.0.1/").expect("Failed to create Redis client");
    let mut con = redis_client
        .get_connection()
        .expect("Failed to get Redis connection");

    let task_to_send = PubSubTask::new(
        "task-from-pubsub".to_string(),
        "do something important".to_string(),
        5,
    );
    let task_json = task_to_send.to_json().unwrap();

    let _: () = redis::cmd("PUBLISH")
        .arg(&channel_name)
        .arg(&task_json)
        .query(&mut con)
        .expect("Failed to publish message");

    // 4. Assert
    // Wait for the listener to process the message and add it to the queue
    thread::sleep(Duration::from_millis(50));

    assert_eq!(worker.queue_len(), 1, "Worker queue should have one task");

    let received_task = {
        let mut queue = worker.queue.lock().unwrap();
        queue
            .pop()
            .expect("Queue was empty when it should have had a task")
    };

    // The process method prints to stdout, which we don't need to verify here.
    // We just need to ensure the correct task was received.
    assert_eq!(received_task, task_to_send);
}
