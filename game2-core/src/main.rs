mod task_queue;
mod manager_bot;
mod agent_kb;

use task_queue::TaskQueue;
use manager_bot::ManagerBot;
use agent_kb::AgentKB;

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

    // Demonstrate Agent KB
    println!("\n=== Agent KB Demo ===");
    let impl_engineer_kb = AgentKB::new("implementation-engineer");
    impl_engineer_kb.init().unwrap();

    println!("\nInitializing Implementation Engineer KB...");
    impl_engineer_kb.add_kb_entry("task-history.md", "Completed task_1: Demonstration task").unwrap();
    impl_engineer_kb.add_kb_entry("patterns-learned.md", "Learned: Use simple line-based format for file storage").unwrap();

    println!("KB entries added.");

    let manager_kb = AgentKB::new("manager-bot");
    manager_kb.init().unwrap();

    println!("\nInitializing Manager Bot KB...");
    manager_kb.add_kb_entry("delegation-history.md", "Assigned task_1 to Implementation Engineer").unwrap();
    manager_kb.add_kb_entry("sprint-history.md", "Sprint 0: Infrastructure setup started").unwrap();

    println!("KB entries added.");

    // List KB files
    println!("\nListing KB files for Implementation Engineer:");
    match impl_engineer_kb.list_kb_files() {
        Ok(files) => {
            for file in files {
                println!("  - {}", file);
            }
        }
        Err(e) => eprintln!("Error: {}", e),
    }

    println!("\n=== Demo Complete ===");
}
