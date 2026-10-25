# Patterns Learned

**Agent:** Implementation Engineer
**Last Updated:** 2026-10-25
**Version:** 0.1.0

## Summary
Code patterns and architectural patterns discovered or implemented in the codebase.

## Entries

### Line-Based File Storage Pattern
**Date Added:** 2026-10-25
**Context:** Implementing task queue without external dependencies
**Content:** For simple data storage, use line-based format instead of JSON/serde. Each record is one line, fields separated by delimiter (e.g., `|`).
**Example:**
```rust
fn format_line(task: &Task) -> String {
    format!("{}|{}|{}|{}|{}", task.id, task.description, task.assigned_to, task.status.as_str(), task.created_at)
}
```
**Source:** Task #2 - Task queue implementation
**Confidence:** High
**Tags:** file-io, serialization, simple-format

### Module Structure Pattern
**Date Added:** 2026-10-25
**Context:** Organizing Rust modules in workspace
**Content:** Keep modules in `src/` directory. Each module should have a single responsibility. Use `mod` declarations in `main.rs` to bring modules into scope.
**Example:**
```rust
// In main.rs
mod task_queue;
mod manager_bot;
mod agent_kb;
```
**Source:** Task #1 - Project initialization
**Confidence:** High
**Tags:** module-organization, rust, architecture

### Agent Knowledge Base Pattern
**Date Added:** 2026-10-25
**Context:** Implementing agent KB system
**Content:** Each agent has `.agents/{agent-name}/kb/` directory. KB files track learnings, patterns, history. Use append-only style for entries to preserve history.
**Example:**
```rust
let kb = AgentKB::new("implementation-engineer");
kb.init().unwrap();
kb.add_kb_entry("task-history.md", "Completed task X").unwrap();
```
**Source:** Task #4 - Agent KB implementation
**Confidence:** High
**Tags:** agent-kb, knowledge-management, architecture
---
**Date:** 1792950811
**Agent:** implementation-engineer

Learned: Use simple line-based format for file storage
