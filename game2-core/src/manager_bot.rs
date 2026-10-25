//! Simple Manager Bot prototype
//!
//! This module provides a basic Manager Bot that can:
//! - Read tasks from the task queue
//! - Assign tasks to agents
//! - Track task completion
//! - Log assignments and completions

use crate::task_queue::{Task, TaskQueue, TaskStatus};
use std::fs::OpenOptions;
use std::io::Write;

/// Manager Bot for task coordination
pub struct ManagerBot {
    task_queue: TaskQueue,
    log_file: String,
}

impl ManagerBot {
    /// Create a new Manager Bot
    pub fn new(task_queue_path: &str, log_file: &str) -> Self {
        Self {
            task_queue: TaskQueue::new(task_queue_path),
            log_file: log_file.to_string(),
        }
    }

    /// Log a message to the log file
    fn log(&self, message: &str) -> Result<(), String> {
        let mut file = OpenOptions::new()
            .write(true)
            .create(true)
            .append(true)
            .open(&self.log_file)
            .map_err(|e| format!("Failed to open log file: {}", e))?;

        writeln!(file, "{}", message)
            .map_err(|e| format!("Failed to write to log: {}", e))?;

        Ok(())
    }

    /// Read pending tasks from the queue
    pub fn read_pending_tasks(&self) -> Result<Vec<Task>, String> {
        let tasks = self.task_queue.list_pending()?;
        self.log(&format!("Read {} pending tasks", tasks.len()))?;
        Ok(tasks)
    }

    /// Assign a task to an agent
    pub fn assign_task(&self, task_id: &str, agent: &str) -> Result<(), String> {
        self.task_queue.assign_task(task_id, agent)?;
        self.log(&format!("Assigned task {} to agent {}", task_id, agent))?;
        Ok(())
    }

    /// Auto-assign all pending tasks to available agents
    /// For this prototype, we'll use a simple round-robin approach
    pub fn auto_assign_tasks(&self, agents: &[&str]) -> Result<Vec<String>, String> {
        let pending = self.read_pending_tasks()?;
        let mut assignments = Vec::new();

        for (i, task) in pending.iter().enumerate() {
            let agent = agents[i % agents.len()];
            self.assign_task(&task.id, agent)?;
            assignments.push(format!("{} -> {}", task.id, agent));
        }

        Ok(assignments)
    }

    /// Mark a task as complete
    pub fn mark_task_complete(&self, task_id: &str) -> Result<(), String> {
        self.task_queue.mark_complete(task_id)?;
        self.log(&format!("Marked task {} as complete", task_id))?;
        Ok(())
    }

    /// Get all tasks (any status)
    pub fn get_all_tasks(&self) -> Result<Vec<Task>, String> {
        self.task_queue.list_all()
    }

    /// Print task summary
    pub fn print_summary(&self) -> Result<(), String> {
        let tasks = self.get_all_tasks()?;

        println!("=== Task Summary ===");
        println!("Total tasks: {}", tasks.len());

        let pending = tasks.iter().filter(|t| t.status == TaskStatus::Pending).count();
        let in_progress = tasks.iter().filter(|t| t.status == TaskStatus::InProgress).count();
        let complete = tasks.iter().filter(|t| t.status == TaskStatus::Complete).count();

        println!("Pending: {}", pending);
        println!("In Progress: {}", in_progress);
        println!("Complete: {}", complete);
        println!("==================");

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn test_manager_bot_creation() {
        let bot = ManagerBot::new("test_manager_tasks.txt", "test_manager.log");
        let _ = fs::remove_file("test_manager_tasks.txt");
        let _ = fs::remove_file("test_manager.log");
        assert!(true); // Just test creation works
    }

    #[test]
    fn test_read_pending_tasks() {
        let _ = fs::remove_file("test_manager_tasks.txt");
        let _ = fs::remove_file("test_manager.log");

        let queue = TaskQueue::new("test_manager_tasks.txt");
        queue.add_task("Test task").unwrap();

        let bot = ManagerBot::new("test_manager_tasks.txt", "test_manager.log");
        let pending = bot.read_pending_tasks().unwrap();
        assert_eq!(pending.len(), 1);

        // Cleanup
        let _ = fs::remove_file("test_manager_tasks.txt");
        let _ = fs::remove_file("test_manager.log");
    }

    #[test]
    fn test_assign_task() {
        let _ = fs::remove_file("test_manager_tasks.txt");
        let _ = fs::remove_file("test_manager.log");

        let queue = TaskQueue::new("test_manager_tasks.txt");
        let task = queue.add_task("Test task").unwrap();

        let bot = ManagerBot::new("test_manager_tasks.txt", "test_manager.log");
        bot.assign_task(&task.id, "Test Agent").unwrap();

        let tasks = bot.get_all_tasks().unwrap();
        let updated_task = tasks.iter().find(|t| t.id == task.id).unwrap();
        assert_eq!(updated_task.assigned_to, "Test Agent");
        assert_eq!(updated_task.status, TaskStatus::InProgress);

        // Cleanup
        let _ = fs::remove_file("test_manager_tasks.txt");
        let _ = fs::remove_file("test_manager.log");
    }

    #[test]
    fn test_auto_assign_tasks() {
        let _ = fs::remove_file("test_manager_tasks.txt");
        let _ = fs::remove_file("test_manager.log");

        let queue = TaskQueue::new("test_manager_tasks.txt");
        queue.add_task("Task 1").unwrap();
        queue.add_task("Task 2").unwrap();
        queue.add_task("Task 3").unwrap();

        let bot = ManagerBot::new("test_manager_tasks.txt", "test_manager.log");
        let agents = vec!["Agent A", "Agent B"];
        let assignments = bot.auto_assign_tasks(&agents).unwrap();

        assert_eq!(assignments.len(), 3);

        // Cleanup
        let _ = fs::remove_file("test_manager_tasks.txt");
        let _ = fs::remove_file("test_manager.log");
    }
}
