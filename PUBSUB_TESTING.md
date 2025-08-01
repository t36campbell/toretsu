## Manual Testing Instructions

To test the Redis Pub/Sub functionality manually:

### 1. Start Redis Server
```bash
redis-server
```

### 2. Run the Example
```bash
cargo run --example pubsub_demo
```

### 3. In Another Terminal, Publish Tasks
```bash
# Publish a simple task
redis-cli PUBLISH tasks '{"id":"manual-1","payload":"Hello from Redis CLI","priority":1}'

# Publish a high priority task
redis-cli PUBLISH urgent_tasks '{"id":"urgent-1","payload":"Critical alert","priority":0}'

# Publish multiple tasks with different priorities
redis-cli PUBLISH tasks '{"id":"data-processing","payload":"Process user analytics","priority":5}'
redis-cli PUBLISH tasks '{"id":"email-send","payload":"Send welcome email to user@example.com","priority":3}'
redis-cli PUBLISH tasks '{"id":"report-gen","payload":"Generate monthly report","priority":7}'
```

### 4. Expected Output
You should see output like:
```
PubSubWorker worker-name received task: manual-1 with payload: Hello from Redis CLI
Processing task manual-1: Hello from Redis CLI
PubSubWorker worker-name received task: urgent-1 with payload: Critical alert
Processing task urgent-1: Critical alert
```

### 5. Test JSON Format
The expected JSON format for tasks is:
```json
{
  "id": "unique-task-id",
  "payload": "Task description or data",
  "priority": 1
}
```

Where priority is an integer (lower numbers = higher priority).

### 6. Multiple Channels
The example listens to both "tasks" and "urgent_tasks" channels. You can publish to either:
- Regular tasks: `PUBLISH tasks '...'`
- Urgent tasks: `PUBLISH urgent_tasks '...'`