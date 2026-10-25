# Agent Knowledge Base Architecture

## Overview

Each agent maintains their own knowledge base (KB) as text files. These KBs are separate from the core research wiki and contain the agent's learnings, patterns, and history. KBs are runtime state - they evolve as agents work and learn.

## Directory Structure

```
.agents/
├── implementation-engineer/
│   └── kb/
│       ├── patterns-learned.md
│       ├── architecture-notes.md
│       ├── common-mistakes.md
│       ├── successful-patterns.md
│       ├── rust-idioms.md
│       └── task-history.md
├── manager-bot/
│   └── kb/
│       ├── agent-capabilities.md
│       ├── delegation-history.md
│       ├── sprint-history.md
│       ├── promotion-log.md
│       ├── systemic-issues.md
│       ├── delegation-heuristics.md
│       └── user-preferences.md
├── fun-inspector/
│   └── kb/
│       ├── fun-signals.md
│       ├── pacing-patterns.md
│       ├── balance-learnings.md
│       ├── design-principles.md
│       ├── player-behavior.md
│       ├── successful-changes.md
│       └── failed-experiments.md
└── [other agents]/
    └── kb/
        └── [agent-specific files]
```

## KB File Formats

### Generic Template

Each KB file should follow this structure:

```markdown
# [KB File Name]

Last Updated: [Date]
Agent: [Agent Name]
Version: [Semantic version for this KB file]

## Summary
[Brief description of what this file tracks]

## Entries

### [Entry 1]
**Date Added:** [Date]
**Context:** [What was happening when this was learned]
**Content:** [The actual knowledge/learning]
**Source:** [How this was learned - task, review, etc.]
**Confidence:** [High/Medium/Low]
**Tags:** [tag1, tag2, tag3]

### [Entry 2]
[Same structure]
```

### Implementation Engineer KB Examples

#### patterns-learned.md
Tracks code patterns found in the codebase.

```markdown
# Patterns Learned

Last Updated: 2026-10-25
Agent: Implementation Engineer
Version: 0.1.0

## Summary
Code patterns discovered in the codebase that should be reused.

## Entries

### ECS Component Pattern
**Date Added:** 2026-10-25
**Context:** Implementing character system
**Content:** Components are structs with data only, no logic. Systems operate on components. Example:
```rust
struct Position { x: f32, y: f32 }
struct Health { current: u32, max: u32 }
// System queries entities with Position and Health
```
**Source:** Task #23 - Character component implementation
**Confidence:** High
**Tags:** ecs, components, architecture

### Error Handling Pattern
**Date Added:** 2026-10-25
**Context:** Implementing resource loading
**Content:** Use `Result<T, E>` with custom error types. Use `thiserror` for error enums. Example:
```rust
#[derive(Debug, thiserror::Error)]
enum LoadError {
    #[error("File not found: {0}")]
    NotFound(PathBuf),
    #[error("Parse error: {0}")]
    Parse(String),
}
```
**Source:** Task #15 - Resource loader
**Confidence:** High
**Tags:** error-handling, rust, patterns
```

#### task-history.md
Tracks completed tasks with outcomes.

```markdown
# Task History

Last Updated: 2026-10-25
Agent: Implementation Engineer
Version: 0.1.0

## Summary
History of tasks completed with outcomes and lessons learned.

## Entries

### Task #23: Character Component System
**Date Completed:** 2026-10-25
**Assigned By:** Manager Bot
**Approach:** Implemented Position, Health, Name components. Used Bevy ECS patterns.
**What Worked:** Component-based approach made testing easy. Clear separation of concerns.
**What Didn't Work:** Initially tried to put logic in components, had to refactor.
**Time Taken:** 4 hours (estimated 3 hours)
**Peer Review Feedback:** "Good separation of concerns. Consider adding component relationships."
**Lessons:** Components should be data-only. Logic goes in systems.
**Outcome:** Success, PR merged
**Confidence:** High
**Tags:** ecs, components, refactoring
```

### Manager Bot KB Examples

#### agent-capabilities.md
Tracks current assessment of each agent's capabilities.

```markdown
# Agent Capabilities

Last Updated: 2026-10-25
Agent: Manager Bot
Version: 0.1.0

## Summary
Current capability assessment for each agent.

## Entries

### Implementation Engineer
**Current Level:** Mid
**Promotion Readiness:** Not ready
**Strengths:**
- Solid Rust knowledge
- Good at following existing patterns
- Reliable for straightforward tasks
**Weaknesses:**
- Struggles with architectural decisions
- Needs guidance on complex features
- Has made 2 mistakes in refactoring
**Recent Performance:**
- Last 5 tasks: 4 success, 1 required rework
- Average time: 1.2x estimate
- Peer reviews: Generally positive
**Recommended For:**
- Simple to medium complexity implementation
- Feature implementation with clear specs
- NOT recommended for: Architectural decisions, complex refactors
**Next Promotion Criteria:** Need 5 more successful complex tasks + positive architectural contribution
**Last Updated:** 2026-10-25
```

#### delegation-heuristics.md
Evolves rules for task assignment.

```markdown
# Delegation Heuristics

Last Updated: 2026-10-25
Agent: Manager Bot
Version: 0.1.0

## Summary
Rules for assigning tasks to agents based on historical success rates.

## Base Rules (Initial)
- Implementation tasks → Implementation Engineer
- Refactoring tasks → Pedantic Senior Engineer
- Testing tasks → Appropriate tester based on type
- Research tasks → Appropriate researcher based on domain

## Evolved Rules (Learned)

### Implementation Tasks
**Simple Implementation** (estimated < 4 hours, clear specs)
- Assign to: Implementation Engineer (Mid or higher)
- Success rate: 92%
- Example: Adding a new component type, simple UI element

**Complex Implementation** (estimated > 4 hours or unclear specs)
- Assign to: Implementation Engineer (Senior or higher)
- Success rate: 78%
- Example: New system architecture, integration across modules

**Performance-Critical Implementation**
- Assign to: Implementation Engineer (Staff) + Performance Engineer review
- Success rate: 85%
- Example: Rendering optimizations, simulation hot paths

### Refactoring Tasks
**Small Refactors** (single module)
- Assign to: Implementation Engineer (Senior)
- Success rate: 88%
- Example: Extracting a function, renaming for clarity

**Large Refactors** (multiple modules, architectural changes)
- Assign to: Pedantic Senior Engineer
- Success rate: 95%
- Example: Breaking circular dependencies, introducing new pattern

### Testing Tasks
**Unit Test Writing**
- Assign to: Implementation Engineer (who wrote the code)
- Success rate: 90%
- Note: Tests should be written alongside implementation

**Integration Testing**
- Assign to: Exploratory Tester (Data Scientist)
- Success rate: 85%
- Note: Focus on data flow and system interactions

**UI Testing**
- Assign to: Exploratory Tester (UI Clicker)
- Success rate: 82%
- Note: Focus on user flows and edge cases

**Fun/Playability Testing**
- Assign to: Fun Inspector
- Success rate: Unknown (new agent)
- Note: Qualitative assessment, requires game design knowledge

## Confidence Levels
- High: Rule based on 10+ successful assignments
- Medium: Rule based on 5-10 assignments
- Low: Rule based on <5 assignments (experimental)
```

## KB Update Process

### When to Update KB
1. **After Task Completion** - Agent updates their task-history.md
2. **When Learning New Pattern** - Agent updates patterns-learned.md or similar
3. **When Making Mistake** - Agent updates common-mistakes.md
4. **After Peer Review** - Agent incorporates feedback into relevant KB file
5. **Weekly** - Manager Bot reviews all agent KBs and updates agent-capabilities.md
6. **After Sprint** - Manager Bot updates sprint-history.md and delegation-heuristics.md

### KB Update Workflow
1. Agent completes work
2. Agent reflects on what was learned
3. Agent updates relevant KB file(s)
4. Agent commits KB changes to their worktree
5. Agent submits PR (KB changes can be in same PR as work, or separate)
6. Manager Bot reviews KB changes during sprint retrospective
7. Manager Bot updates its own KB based on agent KBs

### KB Versioning
- Each KB file has a semantic version
- Increment major version for structural changes
- Increment minor version for new entries
- Increment patch version for corrections
- This helps track when KB structure changed vs just new content

## KB Quality Rules

### For Agents
1. **Be Specific** - Don't say "learned about ECS". Say "learned that components should be data-only, logic in systems".
2. **Include Context** - When was this learned? What was the situation?
3. **Mark Confidence** - High/Medium/Low. Low confidence entries should be re-evaluated.
4. **Use Tags** - Make entries searchable and groupable.
5. **Date Everything** - When was this learned? When was this last updated?

### For Manager Bot
1. **Review Regularly** - Check agent KBs weekly or after each sprint.
2. **Look for Patterns** - Are multiple agents learning the same thing? Should this be shared?
3. **Update Delegation** - If agent capabilities changed, update delegation heuristics.
4. **Flag for Promotion** - If KB shows readiness, flag for promotion assessment.
5. **Share Insights** - If one agent learned something useful, share with others.

## KB vs Research Wiki

### Research Wiki (`wiki/`)
- **Purpose:** Long-term project knowledge
- **Audience:** Humans and agents
- **Content:** Game design, research, architecture decisions, inspirational material
- **Update Frequency:** As new research is done
- **Scope:** Project-wide, permanent

### Agent KBs (`.agents/*/kb/`)
- **Purpose:** Agent runtime learning and state
- **Audience:** Primarily agents, some human visibility
- **Content:** Agent-specific learnings, patterns, history, capabilities
- **Update Frequency:** After each task, weekly reviews
- **Scope:** Agent-specific, evolving

### Relationship
- Agents may read from Research Wiki to inform their work
- Agents may propose additions to Research Wiki based on their learnings
- Manager Bot may surface agent KB insights to Research Wiki
- Research Wiki informs agent capabilities (e.g., "we decided to use ECS, so Implementation Engineer should learn ECS patterns")

## KB as Test Files

As you mentioned, KBs can serve as test files for agent learning:

### Self-Testing
- Agent can query their own KB: "What have I learned about X?"
- Agent can validate: "Does my KB entry match my actual behavior?"
- Manager Bot can test: "Agent claims capability Y, does their KB support this?"

### Cross-Agent Testing
- Manager Bot can query: "What has Agent A learned that Agent B should know?"
- Manager Bot can share: "Agent A learned pattern X, sharing with Agent B"

### Promotion Testing
- Before promotion, Manager Bot reviews KB: "Does KB demonstrate next-level capability?"
- After promotion, track: "Is KB growing with new capabilities?"

## KB Backup and Recovery

### Backup Strategy
- KBs are text files in git (ignored by .gitignore for `.agents/`? No, should be tracked)
- Actually: KBs should be tracked in git for version history
- `.agents/` directory should be in git, not .gitignore
- Only temporary/runtime state should be excluded

### Recovery
- If KB is corrupted or lost, can restore from git history
- Manager Bot can reconstruct agent-capabilities.md from task history if needed
- KB versioning helps identify when corruption occurred

## KB Privacy and Security

### What's in KB
- Agent learnings (code patterns, design decisions)
- Task history (what was done, outcomes)
- Capability assessments (agent skill levels)
- No secrets, credentials, or sensitive data

### Access Control
- All agents can read all KBs (for learning from each other)
- Agents can only write their own KB (Manager Bot can write all)
- Humans can read all KBs (for transparency)
- KBs are committed to git (for version control and backup)

## Future Enhancements

### KB as Training Data
- KB entries could be used to fine-tune agent prompts
- Historical success patterns could inform future decisions
- Agent "personality" could evolve based on KB content

### KB Search and Retrieval
- Implement search across all KBs
- Agent can ask: "What have we learned about X across all agents?"
- Manager Bot can identify knowledge gaps

### KB-Driven Agent Evolution
- Agents could automatically update their prompts based on KB content
- KB could inform skill selection (e.g., "I've learned a lot about ECS, enable ECS skill")
- KB could drive career ladder progression automatically
