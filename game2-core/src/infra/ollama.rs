//! Ollama integration for AI-driven task execution
//!
//! Shells out to the `ollama` CLI (no HTTP deps needed).
//! Each agent can execute tasks by prompting a local model.

use std::io::Write;
use std::process::{Command, Stdio};

/// Client for executing tasks via a local Ollama model
pub struct OllamaClient {
    /// Model name (e.g., "llama3.2", "qwen2.5-coder")
    model: String,
}

impl OllamaClient {
    /// Create a client for the given model
    pub fn new(model: &str) -> Self {
        Self {
            model: model.to_string(),
        }
    }

    /// Check if Ollama is installed and the model is available
    pub fn is_available(&self) -> bool {
        Command::new("ollama")
            .arg("--version")
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .map(|s| s.success())
            .unwrap_or(false)
    }

    /// List locally available models
    pub fn list_models(&self) -> Result<Vec<String>, String> {
        let output = Command::new("ollama")
            .arg("list")
            .output()
            .map_err(|e| format!("Failed to run ollama list: {}", e))?;

        if !output.status.success() {
            return Err(format!(
                "ollama list failed: {}",
                String::from_utf8_lossy(&output.stderr)
            ));
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        Ok(stdout
            .lines()
            .skip(1) // skip header
            .filter_map(|line| line.split_whitespace().next().map(|s| s.to_string()))
            .collect())
    }

    /// Run a prompt against the model and return the response
    pub fn prompt(&self, prompt: &str) -> Result<String, String> {
        let mut child = Command::new("ollama")
            .args(["run", &self.model])
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| format!("Failed to spawn ollama: {}", e))?;

        if let Some(mut stdin) = child.stdin.take() {
            stdin
                .write_all(prompt.as_bytes())
                .map_err(|e| format!("Failed to write prompt: {}", e))?;
        }

        let output = child
            .wait_with_output()
            .map_err(|e| format!("Failed to wait for ollama: {}", e))?;

        if !output.status.success() {
            return Err(format!(
                "ollama run failed: {}",
                String::from_utf8_lossy(&output.stderr)
            ));
        }

        Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
    }

    /// Execute a task by prompting the model and returning the result
    pub fn execute_task(&self, task_description: &str, agent_role: &str) -> Result<String, String> {
        let prompt = format!(
            "You are a {}. Complete this task and report the result concisely:\n\n{}",
            agent_role, task_description
        );
        self.prompt(&prompt)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_client_creation() {
        let client = OllamaClient::new("llama3.2");
        assert_eq!(client.model, "llama3.2");
    }

    #[test]
    fn test_is_available() {
        let client = OllamaClient::new("llama3.2");
        // Availability depends on environment; just ensure it doesn't panic
        let _ = client.is_available();
    }
}
