//! Implementation Engineer agent: executes code tasks.
//! Core agent that does the actual work - writes code, fixes bugs, builds features.

use crate::infra::ollama::OllamaClient;

/// An implementation task assigned to this agent
#[derive(Debug)]
pub struct ImplementationTask {
    pub description: String,
    pub task_type: String,  // feature, bugfix, refactor, test
    pub context: String,    // relevant files/code
}

/// Result of executing a task
#[derive(Debug)]
pub struct TaskResult {
    pub task_id: String,
    pub success: bool,
    pub output: String,
    pub files_changed: Vec<String>,
}

/// The Implementation Engineer agent
pub struct ImplementationEngineer {
    pub name: String,
    pub skill_level: u32,  // 1=junior, 2=mid, 3=senior, 4=staff
    pub client: OllamaClient,
}

impl ImplementationEngineer {
    pub fn new(model: &str) -> Self {
        Self {
            name: "Implementation Engineer".into(),
            skill_level: 2, // mid-level
            client: OllamaClient::new(model),
        }
    }

    /// Execute a task via the LLM
    pub fn execute(&self, task: &ImplementationTask) -> Result<TaskResult, String> {
        let prompt = format!(
            "You are an Implementation Engineer (level {}). Complete this {} task:\n\n{}\n\nContext:\n{}\n\nReport: what you did, files changed, result.",
            self.skill_level, task.task_type, task.description, task.context
        );
        let output = self.client.prompt(&prompt)?;
        Ok(TaskResult {
            task_id: String::new(),
            success: true,
            output,
            files_changed: vec![],
        })
    }

    /// Can this agent handle the task?
    pub fn can_handle(&self, task_type: &str) -> bool {
        matches!(task_type, "feature" | "bugfix" | "refactor" | "test" | "code-implementation")
    }
}
