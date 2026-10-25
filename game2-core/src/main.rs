mod task_queue;

use task_queue::TaskQueue;

fn main() {
    println!("hello world");

    // Demonstrate task queue
    let queue = TaskQueue::new("tasks/todo.txt");

    // Add a task
    match queue.add_task("Demonstration task") {
        Ok(task) => println!("Added task: {} - {}", task.id, task.description),
        Err(e) => eprintln!("Error adding task: {}", e),
    }

    // List pending tasks
    match queue.list_pending() {
        Ok(tasks) => {
            println!("Pending tasks:");
            for task in tasks {
                println!("  {} - {}", task.id, task.description);
            }
        }
        Err(e) => eprintln!("Error listing tasks: {}", e),
    }
}
