use serde::{Deserialize, Serialize};
use toretsu::{client::Client, task::Task, worker::Worker};

#[derive(Clone, Copy, Eq, Ord, PartialEq, PartialOrd, Serialize, Deserialize, Debug)]
struct Job {
    id: u32,
    priority: u32,
    data: [u8; 4],
}

impl Job {
    fn new(id: u32, priority: u32, data: [u8; 4]) -> Self {
        Self { id, priority, data }
    }
}

impl Task for Job {
    fn process(&mut self) {
        println!(
            "Processing job {} with priority {} and data {:?}",
            self.id, self.priority, self.data
        );
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Toretsu Redis Backup Example");
    println!("============================");

    // This example shows how backup/restore would work with a Redis connection
    // Note: This requires a running Redis instance

    println!("\nAttempting to connect to Redis...");
    // Note: Client::new() will panic if it cannot connect to Redis.
    // Ensure Redis is running before executing this example.
    let mut client = Client::new();
    println!("Connected to Redis successfully!");

    // Create some sample jobs
    let jobs = vec![
        Job::new(1, 10, [1, 2, 3, 4]),
        Job::new(2, 5, [5, 6, 7, 8]),
        Job::new(3, 15, [9, 10, 11, 12]),
        Job::new(4, 8, [13, 14, 15, 16]),
    ];

    // Create a worker with the jobs
    let worker = Worker::from(jobs);
    let queue_id = worker.queue.id;

    println!("Created worker with queue ID: {}", queue_id);
    println!("Queue contains {} jobs", worker.queue.len());

    // Backup the queue
    println!("Backing up queue to Redis...");
    worker.backup_queue(&mut client)?;
    println!("Queue backed up successfully!");

    // Simulate worker failure and restoration
    println!("\nSimulating worker failure...");
    drop(worker);

    // Create a new worker and restore from backup
    println!("Creating new worker and restoring from backup...");
    let mut restored_worker = Worker::<Job>::new_with_restore(queue_id, &mut client);

    println!(
        "Restored worker has {} jobs in queue",
        restored_worker.queue.len()
    );

    // Process all jobs
    println!("Processing jobs from restored worker...");
    restored_worker.clock_in();

    // Clean up - delete the backup
    println!("\nCleaning up backup from Redis...");
    restored_worker.delete_queue_backup(&mut client)?;
    println!("Backup deleted from Redis");

    Ok(())
}
