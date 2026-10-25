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

    // Add some tasks (with dependencies to demonstrate)
    println!("Adding tasks...");
    let t1 = queue.add_task("Design ECS architecture").unwrap();
    let t2 = queue.add_task_with_dependencies("Build ECS component system", &[t1.id.clone()]).unwrap();
    let t3 = queue.add_task_with_dependencies("Implement entity manager", &[t2.id.clone()]).unwrap();
    let _t4 = queue.add_task_with_dependencies("Write integration tests", &[t3.id.clone()]).unwrap();

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

    // Auto-assign tasks using capability matching
    println!("\nAuto-assigning tasks (capability-based)...");
    let agents = vec![]; // Agents not needed for capability-based assignment
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

    // Demonstrate End-to-End Workflow
    println!("\n=== End-to-End Workflow Demo ===");

    // Step 1: Create a new task
    println!("\nStep 1: Creating new task...");
    let task = queue.add_task("Add a comment to main.rs").unwrap();
    println!("Created task: {} - {}", task.id, task.description);

    // Step 2: Manager Bot assigns the task
    println!("\nStep 2: Manager Bot assigning task...");
    manager.assign_task(&task.id, "Implementation Engineer").unwrap();
    println!("Assigned to: Implementation Engineer");

    // Step 3: Implementation Engineer "executes" the task
    println!("\nStep 3: Implementation Engineer executing task...");
    // Simulate task execution by just waiting a bit and logging
    std::thread::sleep(std::time::Duration::from_secs(1));
    println!("Task executed (simulated).");

    // Step 4: Implementation Engineer updates KB
    println!("\nStep 4: Updating Implementation Engineer KB...");
    impl_engineer_kb.add_kb_entry(
        "task-history.md",
        &format!("Completed task {}: Add a comment to main.rs", task.id)
    ).unwrap();
    impl_engineer_kb.add_kb_entry(
        "patterns-learned.md",
        "Learned: End-to-end workflow from task creation to KB update"
    ).unwrap();
    println!("KB updated.");

    // Step 5: Manager Bot marks task complete and updates KB
    println!("\nStep 5: Manager Bot marking task complete...");
    manager.mark_task_complete(&task.id).unwrap();
    println!("Task marked complete.");

    println!("\nStep 6: Updating Manager Bot KB...");
    manager_kb.add_kb_entry(
        "delegation-history.md",
        &format!("Assigned task {} to Implementation Engineer, marked complete", task.id)
    ).unwrap();
    manager_kb.add_kb_entry(
        "sprint-history.md",
        "Sprint 0: End-to-end workflow demonstrated successfully"
    ).unwrap();
    println!("KB updated.");

    // Print final summary
    println!();
    manager.print_summary().unwrap();

    println!("\n=== Workflow Demo Complete ===");
}
