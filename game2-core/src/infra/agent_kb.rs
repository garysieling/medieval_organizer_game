//! Agent Knowledge Base implementation
//!
//! This module provides basic KB read/write functionality for agents.
//! Each agent has their own KB directory structure with various files
//! for tracking learnings, patterns, history, and capabilities.

use std::fs::{self, File, OpenOptions};
use std::io::{BufRead, BufReader, BufWriter, Write};
use std::path::Path;

/// Agent Knowledge Base
pub struct AgentKB {
    agent_name: String,
    kb_dir: String,
}

impl AgentKB {
    /// Create a new Agent KB for the given agent
    pub fn new(agent_name: &str) -> Self {
        let kb_dir = format!(".agents/{}/kb", agent_name);
        Self {
            agent_name: agent_name.to_string(),
            kb_dir,
        }
    }

    /// Create a new Agent KB with a custom root directory (for tests)
    pub fn with_root(agent_name: &str, root: &str) -> Self {
        let kb_dir = format!("{}/{}/kb", root, agent_name);
        Self {
            agent_name: agent_name.to_string(),
            kb_dir,
        }
    }

    /// Initialize the KB directory structure if it doesn't exist
    pub fn init(&self) -> Result<(), String> {
        fs::create_dir_all(&self.kb_dir)
            .map_err(|e| format!("Failed to create KB directory: {}", e))?;
        Ok(())
    }

    /// Get the path to a specific KB file
    fn kb_file_path(&self, filename: &str) -> String {
        format!("{}/{}", self.kb_dir, filename)
    }

    /// Read a KB file
    pub fn read_kb_file(&self, filename: &str) -> Result<String, String> {
        let path = self.kb_file_path(filename);
        if !Path::new(&path).exists() {
            return Ok(String::new());
        }

        let file = File::open(&path)
            .map_err(|e| format!("Failed to open KB file {}: {}", filename, e))?;
        let reader = BufReader::new(file);
        let mut content = String::new();
        for line in reader.lines() {
            let line = line.map_err(|e| format!("Failed to read line: {}", e))?;
            content.push_str(&line);
            content.push('\n');
        }
        Ok(content)
    }

    /// Write to a KB file (overwrites existing content)
    pub fn write_kb_file(&self, filename: &str, content: &str) -> Result<(), String> {
        let path = self.kb_file_path(filename);
        let file = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(&path)
            .map_err(|e| format!("Failed to open KB file {} for writing: {}", filename, e))?;
        let mut writer = BufWriter::new(file);
        writer.write_all(content.as_bytes())
            .map_err(|e| format!("Failed to write to KB file: {}", e))?;
        writer.flush()
            .map_err(|e| format!("Failed to flush KB file: {}", e))?;
        Ok(())
    }

    /// Append to a KB file
    pub fn append_kb_file(&self, filename: &str, content: &str) -> Result<(), String> {
        let path = self.kb_file_path(filename);
        let file = OpenOptions::new()
            .write(true)
            .create(true)
            .append(true)
            .open(&path)
            .map_err(|e| format!("Failed to open KB file {} for appending: {}", filename, e))?;
        let mut writer = BufWriter::new(file);
        writer.write_all(content.as_bytes())
            .map_err(|e| format!("Failed to append to KB file: {}", e))?;
        writer.flush()
            .map_err(|e| format!("Failed to flush KB file: {}", e))?;
        Ok(())
    }

    /// Add an entry to a KB file (adds with timestamp)
    pub fn add_kb_entry(&self, filename: &str, entry: &str) -> Result<(), String> {
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        let entry_with_meta = format!(
            "---\n**Date:** {}\n**Agent:** {}\n\n{}\n",
            timestamp, self.agent_name, entry
        );
        self.append_kb_file(filename, &entry_with_meta)
    }

    /// Check if a KB file exists
    pub fn kb_file_exists(&self, filename: &str) -> bool {
        Path::new(&self.kb_file_path(filename)).exists()
    }

    /// List all KB files for this agent
    pub fn list_kb_files(&self) -> Result<Vec<String>, String> {
        if !Path::new(&self.kb_dir).exists() {
            return Ok(Vec::new());
        }

        let mut files = Vec::new();
        let entries = fs::read_dir(&self.kb_dir)
            .map_err(|e| format!("Failed to read KB directory: {}", e))?;

        for entry in entries {
            let entry = entry.map_err(|e| format!("Failed to read entry: {}", e))?;
            let path = entry.path();
            if path.is_file() {
                if let Some(filename) = path.file_name().and_then(|n| n.to_str()) {
                    files.push(filename.to_string());
                }
            }
        }
        Ok(files)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn tmp_kb(name: &str) -> (AgentKB, String) {
        let root = std::env::temp_dir().join(format!("kb_{}", name));
        let root = root.to_str().unwrap().to_string();
        let _ = fs::remove_dir_all(&root);
        (AgentKB::with_root("test-agent", &root), root)
    }

    #[test]
    fn test_kb_initialization() {
        let (kb, root) = tmp_kb("init");
        kb.init().unwrap();
        assert!(Path::new(&format!("{}/test-agent/kb", root)).exists());
        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn test_write_and_read_kb_file() {
        let (kb, root) = tmp_kb("rw");
        kb.init().unwrap();

        let content = "# Test KB\n\nThis is a test.";
        kb.write_kb_file("test.md", content).unwrap();
        let read_content = kb.read_kb_file("test.md").unwrap();
        assert_eq!(read_content, format!("{}\n", content));

        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn test_add_kb_entry() {
        let (kb, root) = tmp_kb("entries");
        kb.init().unwrap();

        kb.add_kb_entry("entries.md", "First entry").unwrap();
        kb.add_kb_entry("entries.md", "Second entry").unwrap();

        let content = kb.read_kb_file("entries.md").unwrap();
        assert!(content.contains("First entry"));
        assert!(content.contains("Second entry"));
        assert!(content.contains("test-agent"));

        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn test_list_kb_files() {
        let (kb, root) = tmp_kb("list");
        kb.init().unwrap();

        kb.write_kb_file("file1.md", "Content 1").unwrap();
        kb.write_kb_file("file2.md", "Content 2").unwrap();

        let files = kb.list_kb_files().unwrap();
        assert_eq!(files.len(), 2);
        assert!(files.contains(&"file1.md".to_string()));
        assert!(files.contains(&"file2.md".to_string()));

        let _ = fs::remove_dir_all(&root);
    }
}
