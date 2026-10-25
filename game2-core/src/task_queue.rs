//! Simple task queue implementation
//!
//! This module provides a basic task queue backed by a text file.
//! Tasks are stored in a simple line-based format for simplicity.

use std::fs::{File, OpenOptions};
use std::io::{BufRead, BufReader, BufWriter, Write};
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

/// Represents a single task in the queue
#[derive(Debug, Clone)]
pub struct Task {
    /// Unique identifier for the task
    pub id: String,
    /// Task description
    pub description: String,
    /// Assigned agent (or "unassigned")
    pub assigned_to: String,
    /// Task status
    pub status: TaskStatus,
    /// Creation timestamp (Unix epoch)
    pub created_at: u64,
}

/// Task status
#[derive(Debug, Clone, PartialEq)]
pub enum TaskStatus {
    Pending,
    InProgress,
    Complete,
}

impl TaskStatus {
    fn from_str(s: &str) -> Result<Self, String> {
        match s {
            "Pending" => Ok(TaskStatus::Pending),
            "InProgress" => Ok(TaskStatus::InProgress),
            "Complete" => Ok(TaskStatus::Complete),
            _ => Err(format!("Invalid status: {}", s)),
        }
    }

    fn as_str(&self) -> &'static str {
        match self {
            TaskStatus::Pending => "Pending",
            TaskStatus::InProgress => "InProgress",
            TaskStatus::Complete => "Complete",
        }
    }
}

/// Task queue backed by a file
pub struct TaskQueue {
    file_path: String,
}

impl TaskQueue {
    /// Create a new task queue with the given file path
    pub fn new(file_path: &str) -> Self {
        Self {
            file_path: file_path.to_string(),
        }
    }

    /// Load all tasks from the file
    pub fn load_tasks(&self) -> Result<Vec<Task>, String> {
        if !Path::new(&self.file_path).exists() {
            return Ok(Vec::new());
        }

        let file = File::open(&self.file_path)
            .map_err(|e| format!("Failed to open task file: {}", e))?;
        let reader = BufReader::new(file);
        let mut tasks = Vec::new();

        for line in reader.lines() {
            let line = line.map_err(|e| format!("Failed to read line: {}", e))?;
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            let task = Self::parse_line(&line)?;
            tasks.push(task);
        }
        Ok(tasks)
    }

    /// Parse a single task line
    fn parse_line(line: &str) -> Result<Task, String> {
        // Format: id|description|assigned_to|status|created_at
        let parts: Vec<&str> = line.split('|').collect();
        if parts.len() != 5 {
            return Err(format!("Invalid task format: {}", line));
        }

        let id = parts[0].to_string();
        let description = parts[1].to_string();
        let assigned_to = parts[2].to_string();
        let status = TaskStatus::from_str(parts[3])?;
        let created_at = parts[4].parse::<u64>()
            .map_err(|e| format!("Invalid timestamp: {}", e))?;

        Ok(Task {
            id,
            description,
            assigned_to,
            status,
            created_at,
        })
    }

    /// Format a task as a line
    fn format_line(task: &Task) -> String {
        format!(
            "{}|{}|{}|{}|{}",
            task.id,
            task.description,
            task.assigned_to,
            task.status.as_str(),
            task.created_at
        )
    }

    /// Save all tasks to the file
    fn save_tasks(&self, tasks: &[Task]) -> Result<(), String> {
        let file = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(&self.file_path)
            .map_err(|e| format!("Failed to open task file for writing: {}", e))?;
        let mut writer = BufWriter::new(file);

        for task in tasks {
            writeln!(writer, "{}", Self::format_line(task))
                .map_err(|e| format!("Failed to write task: {}", e))?;
        }

        writer.flush()
            .map_err(|e| format!("Failed to flush file: {}", e))?;
        Ok(())
    }

    /// Get current timestamp
    fn now() -> u64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs()
    }

    /// Add a new task to the queue
    pub fn add_task(&self, description: &str) -> Result<Task, String> {
        let mut tasks = self.load_tasks()?;
        let id = format!("task_{}", tasks.len() + 1);
        let task = Task {
            id: id.clone(),
            description: description.to_string(),
            assigned_to: "unassigned".to_string(),
            status: TaskStatus::Pending,
            created_at: Self::now(),
        };
        tasks.push(task.clone());
        self.save_tasks(&tasks)?;
        Ok(task)
    }

    /// List all pending tasks
    pub fn list_pending(&self) -> Result<Vec<Task>, String> {
        let tasks = self.load_tasks()?;
        Ok(tasks.into_iter().filter(|t| t.status == TaskStatus::Pending).collect())
    }

    /// List all tasks (any status)
    pub fn list_all(&self) -> Result<Vec<Task>, String> {
        self.load_tasks()
    }

    /// Mark a task as complete
    pub fn mark_complete(&self, task_id: &str) -> Result<(), String> {
        let mut tasks = self.load_tasks()?;
        let task = tasks.iter_mut()
            .find(|t| t.id == task_id)
            .ok_or_else(|| format!("Task not found: {}", task_id))?;
        task.status = TaskStatus::Complete;
        self.save_tasks(&tasks)?;
        Ok(())
    }

    /// Assign a task to an agent
    pub fn assign_task(&self, task_id: &str, agent: &str) -> Result<(), String> {
        let mut tasks = self.load_tasks()?;
        let task = tasks.iter_mut()
            .find(|t| t.id == task_id)
            .ok_or_else(|| format!("Task not found: {}", task_id))?;
        task.assigned_to = agent.to_string();
        task.status = TaskStatus::InProgress;
        self.save_tasks(&tasks)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn test_add_task() {
        let queue = TaskQueue::new("test_tasks.txt");
        let _ = fs::remove_file("test_tasks.txt"); // Clean up if exists

        let task = queue.add_task("Test task").unwrap();
        assert_eq!(task.description, "Test task");
        assert_eq!(task.status, TaskStatus::Pending);
        assert_eq!(task.assigned_to, "unassigned");

        // Cleanup
        let _ = fs::remove_file("test_tasks.txt");
    }

    #[test]
    fn test_list_pending() {
        let queue = TaskQueue::new("test_tasks.txt");
        let _ = fs::remove_file("test_tasks.txt");

        queue.add_task("Task 1").unwrap();
        queue.add_task("Task 2").unwrap();

        let pending = queue.list_pending().unwrap();
        assert_eq!(pending.len(), 2);

        // Cleanup
        let _ = fs::remove_file("test_tasks.txt");
    }

    #[test]
    fn test_mark_complete() {
        let queue = TaskQueue::new("test_tasks.txt");
        let _ = fs::remove_file("test_tasks.txt");

        let task = queue.add_task("Test task").unwrap();
        queue.mark_complete(&task.id).unwrap();

        let pending = queue.list_pending().unwrap();
        assert_eq!(pending.len(), 0);

        // Cleanup
        let _ = fs::remove_file("test_tasks.txt");
    }

    #[test]
    fn test_assign_task() {
        let queue = TaskQueue::new("test_tasks.txt");
        let _ = fs::remove_file("test_tasks.txt");

        let task = queue.add_task("Test task").unwrap();
        queue.assign_task(&task.id, "Implementation Engineer").unwrap();

        let tasks = queue.list_all().unwrap();
        let updated_task = tasks.iter().find(|t| t.id == task.id).unwrap();
        assert_eq!(updated_task.assigned_to, "Implementation Engineer");
        assert_eq!(updated_task.status, TaskStatus::InProgress);

        // Cleanup
        let _ = fs::remove_file("test_tasks.txt");
    }
}
