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
    /// IDs of tasks this task depends on
    pub dependencies: Vec<String>,
    /// Priority: 0=low, 1=normal, 2=high, 3=critical
    pub priority: u8,
    /// Optional deadline (tick)
    pub deadline: Option<u64>,
    /// Preferred agent (empty = any)
    pub preferred_agent: String,
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
        // Format: id|description|assigned_to|status|created_at|deps|priority|deadline|preferred_agent
        // Fields 7-9 optional for backwards compat (5/6-field legacy formats accepted)
        let parts: Vec<&str> = line.split('|').collect();
        if parts.len() < 5 || parts.len() > 9 {
            return Err(format!("Invalid task format: {}", line));
        }

        let id = parts[0].to_string();
        let description = parts[1].to_string();
        let assigned_to = parts[2].to_string();
        let status = TaskStatus::from_str(parts[3])?;
        let created_at = parts[4].parse::<u64>()
            .map_err(|e| format!("Invalid timestamp: {}", e))?;
        let dependencies: Vec<String> = if parts.len() >= 6 && !parts[5].is_empty() {
            parts[5].split(',').map(|s| s.to_string()).collect()
        } else {
            Vec::new()
        };
        let priority: u8 = parts.get(6).and_then(|p| p.parse().ok()).unwrap_or(1);
        let deadline: Option<u64> = parts.get(7).and_then(|p| p.parse().ok());
        let preferred_agent: String = parts.get(8).map(|s| s.to_string()).unwrap_or_default();

        Ok(Task {
            id,
            description,
            assigned_to,
            status,
            created_at,
            dependencies,
            priority,
            deadline,
            preferred_agent,
        })
    }

    /// Format a task as a line
    fn format_line(task: &Task) -> String {
        format!(
            "{}|{}|{}|{}|{}|{}|{}|{}|{}",
            task.id,
            task.description,
            task.assigned_to,
            task.status.as_str(),
            task.created_at,
            task.dependencies.join(","),
            task.priority,
            task.deadline.map(|d| d.to_string()).unwrap_or_default(),
            task.preferred_agent
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
        self.add_task_with_dependencies(description, &[])
    }

    /// Add a new task with dependencies to the queue
    pub fn add_task_with_dependencies(&self, description: &str, dependencies: &[String]) -> Result<Task, String> {
        let mut tasks = self.load_tasks()?;
        let id = format!("task_{}", tasks.len() + 1);
        let task = Task {
            id: id.clone(),
            description: description.to_string(),
            assigned_to: "unassigned".to_string(),
            status: TaskStatus::Pending,
            created_at: Self::now(),
            dependencies: dependencies.to_vec(),
            priority: 1,
            deadline: None,
            preferred_agent: String::new(),
        };
        tasks.push(task.clone());
        self.save_tasks(&tasks)?;
        Ok(task)
    }

    /// Check if a task's dependencies are all complete
    pub fn are_dependencies_met(&self, task_id: &str) -> Result<bool, String> {
        let tasks = self.load_tasks()?;
        let task = tasks.iter()
            .find(|t| t.id == task_id)
            .ok_or_else(|| format!("Task not found: {}", task_id))?;

        for dep_id in &task.dependencies {
            let dep = tasks.iter().find(|t| t.id == *dep_id);
            match dep {
                Some(d) if d.status == TaskStatus::Complete => {}
                _ => return Ok(false),
            }
        }
        Ok(true)
    }

    /// List pending tasks whose dependencies are all complete, sorted by priority (high first)
    pub fn list_ready(&self) -> Result<Vec<Task>, String> {
        let tasks = self.load_tasks()?;
        let mut ready: Vec<Task> = tasks
            .iter()
            .filter(|t| t.status == TaskStatus::Pending)
            .filter(|t| {
                t.dependencies.iter().all(|dep_id| {
                    tasks
                        .iter()
                        .find(|d| &d.id == dep_id)
                        .map(|d| d.status == TaskStatus::Complete)
                        .unwrap_or(false)
                })
            })
            .cloned()
            .collect();
        // Sort by priority (high first), then creation time
        ready.sort_by(|a, b| b.priority.cmp(&a.priority).then(a.created_at.cmp(&b.created_at)));
        Ok(ready)
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
