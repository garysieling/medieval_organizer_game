# Agent KB Module

## Overview

Agent Knowledge Base (KB) system for tracking agent learnings, patterns, history, and capabilities. Each agent has their own KB directory with multiple files for different types of knowledge.

## Directory Structure

```
.agents/
├── implementation-engineer/
│   └── kb/
│       ├── task-history.md      # Completed tasks with outcomes
│       ├── patterns-learned.md  # Code/architecture patterns discovered
│       ├── common-mistakes.md   # Mistakes made and lessons learned
│       ├── architecture-notes.md # Notes on codebase architecture
│       ├── successful-patterns.md # Patterns that worked well
│       ├── rust-idioms.md       # Rust-specific idioms discovered
│       └── capabilities.md     # Capability assessment
└── manager-bot/
    └── kb/
        ├── delegation-history.md # Task assignment history
        ├── sprint-history.md    # Sprint outcomes and lessons
        ├── agent-capabilities.md # Current assessment of agent capabilities
        ├── promotion-log.md     # Promotion/demotion history
        ├── systemic-issues.md   # Recurring issues and resolutions
        ├── delegation-heuristics.md # Rules for task assignment
        └── user-preferences.md  # User preferences and settings
```

## API

### AgentKB::new(agent_name: &str) -> AgentKB
Create a new Agent KB for the given agent. Automatically creates `.agents/{agent_name}/kb/` directory path.

### AgentKB::init() -> Result<(), String>
Initialize the KB directory structure if it doesn't exist.

### AgentKB::read_kb_file(filename: &str) -> Result<String, String>
Read a KB file. Returns empty string if file doesn't exist.

### AgentKB::write_kb_file(filename: &str, content: &str) -> Result<(), String>
Write to a KB file (overwrites existing content).

### AgentKB::append_kb_file(filename: &str, content: &str) -> Result<(), String>
Append to a KB file (creates if doesn't exist).

### AgentKB::add_kb_entry(filename: &str, entry: &str) -> Result<(), String>
Add a timestamped entry to a KB file. Format:
```
---
**Date:** {timestamp}
**Agent:** {agent_name}

{entry}
```

### AgentKB::kb_file_exists(filename: &str) -> bool
Check if a KB file exists.

### AgentKB::list_kb_files() -> Result<Vec<String>, String>
List all KB files for this agent.

## KB File Format

Each KB file should follow this structure:

```markdown
# {KB File Name}

**Agent:** {Agent Name}
**Last Updated:** {Date}
**Version:** {Semantic version}

## Summary
[Brief description of what this file tracks]

## Entries

### [Entry 1]
**Date Added:** {Date}
**Context:** {What was happening when this was learned}
**Content:** {The actual knowledge/learning}
**Source:** {How this was learned}
**Confidence:** {High/Medium/Low}
**Tags:** {tag1, tag2, tag3}
```

## Example Usage

```rust
use agent_kb::AgentKB;

fn main() {
    // Create KB for an agent
    let kb = AgentKB::new("implementation-engineer");
    kb.init().unwrap();

    // Add a task completion entry
    kb.add_kb_entry(
        "task-history.md",
        "Completed task_1: Implemented task queue"
    ).unwrap();

    // Add a pattern learned
    kb.add_kb_entry(
        "patterns-learned.md",
        "Use line-based format for simple file storage"
    ).unwrap();

    // List KB files
    let files = kb.list_kb_files().unwrap();
    for file in files {
        println!("KB file: {}", file);
    }

    // Read a KB file
    let content = kb.read_kb_file("task-history.md").unwrap();
    println!("Task history:\n{}", content);
}
```

## KB File Naming Conventions

### Implementation Engineer
- `task-history.md` - All completed tasks with outcomes
- `patterns-learned.md` - Code/architecture patterns
- `common-mistakes.md` - Mistakes and lessons learned
- `architecture-notes.md` - Notes on codebase architecture
- `successful-patterns.md` - Patterns that worked well
- `rust-idioms.md` - Rust-specific idioms
- `capabilities.md` - Capability assessment

### Manager Bot
- `delegation-history.md` - All task assignments
- `sprint-history.md` - Sprint outcomes and lessons
- `agent-capabilities.md` - Current agent capability assessments
- `promotion-log.md` - Promotion/demotion history
- `systemic-issues.md` - Recurring issues and resolutions
- `delegation-heuristics.md` - Rules for task assignment
- `user-preferences.md` - User preferences and settings

## Testing

Unit tests are included in the module. Run with:
```bash
cargo test --package game2-core
```

## Integration with Other Systems

### With Task Queue
- After task completion, add entry to `task-history.md`
- Include: task ID, description, outcome, lessons learned

### With Manager Bot
- Manager Bot can read agent KBs to assess capabilities
- Manager Bot can update `agent-capabilities.md` based on observations
- Agent KBs inform delegation heuristics

### With Sprint Planning
- After each sprint, update `sprint-history.md`
- Include: tasks completed, outcomes, lessons learned
- Inform next sprint's planning

## Future Enhancements

- Add KB versioning for major structural changes
- Add KB search/query functionality
- Add KB export/import for backup
- Add KB validation (ensure format is correct)
- Add KB metrics (track KB growth, patterns learned over time)
- Add KB sharing between agents
- Add KB-driven agent evolution (automatically update prompts based on learnings)
