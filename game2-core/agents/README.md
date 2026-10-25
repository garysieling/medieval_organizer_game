# Task Queue Module

## Overview

Simple task queue implementation backed by a text file. Tasks are stored in a line-based format for simplicity and to avoid external dependencies.

## File Format

Each task is stored as a single line:
```
id|description|assigned_to|status|created_at
```

Example:
```
task_1|Demonstration task|unassigned|Pending|1729857600
task_2|Another task|Implementation Engineer|InProgress|1729857660
```

## API

### TaskQueue::new(file_path: &str) -> TaskQueue
Create a new task queue with the given file path.

### TaskQueue::add_task(description: &str) -> Result<Task, String>
Add a new task to the queue. Returns the created task.

### TaskQueue::list_pending() -> Result<Vec<Task>, String>
List all pending tasks (status = Pending).

### TaskQueue::list_all() -> Result<Vec<Task>, String>
List all tasks regardless of status.

### TaskQueue::mark_complete(task_id: &str) -> Result<(), String>
Mark a task as complete.

### TaskQueue::assign_task(task_id: &str, agent: &str) -> Result<(), String>
Assign a task to an agent and set status to InProgress.

## Example Usage

```rust
use task_queue::TaskQueue;

fn main() {
    let queue = TaskQueue::new("tasks/todo.txt");

    // Add a task
    let task = queue.add_task("Build something").unwrap();

    // Assign to an agent
    queue.assign_task(&task.id, "Implementation Engineer").unwrap();

    // Mark complete
    queue.mark_complete(&task.id).unwrap();

    // List pending tasks
    let pending = queue.list_pending().unwrap();
    for task in pending {
        println!("{} - {}", task.id, task.description);
    }
}
```

## Testing

Unit tests are included in the module. Run with:
```bash
cargo test --package game2-core
```

Note: Tests may be blocked by Windows Application Control policies in some environments.

## Future Enhancements

- Add task priority
- Add task dependencies
- Add task deadlines
- Add task categories/tags
- Consider migrating to a database for better performance at scale
