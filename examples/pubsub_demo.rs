use std::time::Duration;
use toretsu::client::Client;
use toretsu::pubsub_task::PubSubTask;
use toretsu::worker::PubSubWorker;

fn main() {
    println!("Toretsu Redis Pub/Sub Example");
    println!("This example demonstrates how to use Redis Pub/Sub to add tasks to a worker queue.");
    println!("Note: This requires a Redis server running on localhost:6379\n");

    // Create a PubSubWorker
    let mut worker = PubSubWorker::new();
    println!("Created worker with ID: {}", worker.id);

    // Start the worker
    worker.clock_in();
    println!("Worker clocked in and ready to process tasks");

    // Start listening to Redis channels
    let channels = vec!["tasks".to_string(), "urgent_tasks".to_string()];
    println!("Starting to listen on channels: {:?}", channels);
    worker.start_pubsub_listener(channels);

    // Give the listener a moment to start
    std::thread::sleep(Duration::from_millis(100));

    // Simulate publishing tasks via Redis
    println!("\nPublishing tasks via Redis Pub/Sub...");

    // Create a Redis client for publishing
    match Client::try_new() {
        Ok(client) => {
            let mut client = client;

            // Publish some example tasks
            let tasks = vec![
                PubSubTask::new("task-1".to_string(), "Process user data".to_string(), 1),
                PubSubTask::new(
                    "task-2".to_string(),
                    "Send email notification".to_string(),
                    5,
                ),
                PubSubTask::new("task-3".to_string(), "Generate report".to_string(), 3),
                PubSubTask::new(
                    "urgent-1".to_string(),
                    "Critical system alert".to_string(),
                    0,
                ),
            ];

            for task in tasks {
                let json = task.to_json().unwrap();
                let channel = if task.priority == 0 {
                    "urgent_tasks"
                } else {
                    "tasks"
                };

                match client.publish(channel, &json) {
                    Ok(_) => println!("Published task {} to channel {}", task.id, channel),
                    Err(e) => eprintln!("Failed to publish task {}: {}", task.id, e),
                }

                // Small delay between publishes
                std::thread::sleep(Duration::from_millis(100));
            }

            println!("\nTasks published! The worker should be processing them...");

            // Let the worker process tasks for a few seconds
            std::thread::sleep(Duration::from_secs(3));
        }
        Err(e) => {
            println!(
                "Redis connection failed: {}. Make sure Redis is running on localhost:6379",
                e
            );
            println!("You can start Redis with: redis-server");
        }
    }

    // Clock out the worker
    worker.clock_out();
    println!("\nWorker clocked out. Example complete!");

    println!("\nTo manually test this:");
    println!("1. Start Redis: redis-server");
    println!("2. Run this example: cargo run --example pubsub_demo");
    println!("3. In another terminal, publish tasks:");
    println!("   redis-cli PUBLISH tasks '{{\"id\":\"manual-1\",\"payload\":\"Manual task\",\"priority\":1}}'");
}
