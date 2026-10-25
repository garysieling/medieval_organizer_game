mod task_queue;
mod manager_bot;

use task_queue::TaskQueue;
use manager_bot::ManagerBot;

fn main() {
    println!("hello world");

    // Demonstrate task queue
    let queue = TaskQueue::new("tasks/todo.txt");

    // Add some tasks
    println!("Adding tasks...");
    queue.add_task("Build ECS component system").unwrap();
    queue.add_task("Implement entity manager").unwrap();
    queue.add_task("Write integration tests").unwrap();

    // List pending tasks
    println!("\nPending tasks:");
    match queue.list_pending() {
        Ok(tasks) => {
            for task in tasks {
                println!("  {} - {}", task.id, task.description);
            }
        }
        Err(e) => eprintln!("Error listing tasks: {}", e),
    }

    // Demonstrate Manager Bot
    println!("\n=== Manager Bot Demo ===");
    let manager = ManagerBot::new("tasks/todo.txt", "tasks/manager.log");

    // Read pending tasks
    println!("\nReading pending tasks...");
    match manager.read_pending_tasks() {
        Ok(tasks) => println!("Found {} pending tasks", tasks.len()),
        Err(e) => eprintln!("Error: {}", e),
    }

    // Auto-assign tasks to agents
    println!("\nAuto-assigning tasks...");
    let agents = vec!["Implementation Engineer", "Implementation Engineer"];
    match manager.auto_assign_tasks(&agents) {
        Ok(assignments) => {
            println!("Assignments:");
            for assignment in assignments {
                println!("  {}", assignment);
            }
        }
        Err(e) => eprintln!("Error: {}", e),
    }

    // Print summary
    println!();
    manager.print_summary().unwrap();

    println!("\n=== Demo Complete ===");
}
