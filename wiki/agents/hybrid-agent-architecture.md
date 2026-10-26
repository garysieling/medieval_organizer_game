# Hybrid Agent Architecture

**Status:** Implemented (Literature Survey Agent as proof-of-concept)
**Date:** 2026-10-25

## Overview

Agents in this project use a **hybrid architecture** that combines:
1. **Rust infrastructure** - Data structures, KB utilities, fallback heuristics
2. **Skills** - Claude Code skills that provide the intelligence layer
3. **Flexible execution** - Run via Claude, Ollama, or pure Rust

This allows us to balance **quality**, **cost**, and **speed** for different agent tasks.

## Architecture Diagram

```
┌─────────────────────────────────────────────────────────┐
│ USER / MANAGER BOT                                      │
│ "Run a literature survey on Agriculture"               │
└────────────────────┬────────────────────────────────────┘
                     │
        ┌────────────┴────────────┐
        │ Execution Choice        │
        └────────────┬────────────┘
                     │
     ┌───────────────┼───────────────┐
     │               │               │
     ▼               ▼               ▼
┌─────────┐   ┌──────────┐   ┌──────────┐
│ Claude  │   │ Ollama   │   │   Rust   │
│ (skill) │   │ (cheap)  │   │ (fast)   │
│ High $  │   │ Low qual │   │ No intel │
└────┬────┘   └────┬─────┘   └────┬─────┘
     │             │              │
     └─────────────┼──────────────┘
                   │
                   ▼
     ┌─────────────────────────┐
     │ Rust Infrastructure     │
     │ - Data structures       │
     │ - KB read/write         │
     │ - Utilities             │
     └─────────────────────────┘
```

## Components

### 1. Rust Infrastructure (`game2-core/src/agents/*.rs`)

**Purpose:** Provide data structures and utilities that all execution modes can use.

**What it includes:**
- Data structures (`Survey`, `Source`, `MechanicProposal`, etc.)
- KB file read/write utilities (`LiteratureSurveyKB`)
- Enums and type definitions
- Optional: simple heuristics for fallback

**Example (Literature Survey Agent):**
```rust
// Data structures
pub struct Survey {
    pub domain: String,
    pub sources: Vec<Source>,
    pub mechanics: Vec<MechanicProposal>,
    // ...
}

// KB utilities
pub struct LiteratureSurveyKB {
    pub fn read_kb(&self, filename: &str) -> Result<String, String>
    pub fn write_kb(&self, filename: &str, content: &str) -> Result<(), String>
    pub fn add_survey(&self, survey: &Survey) -> Result<(), String>
    // ...
}
```

### 2. Skills (`.claude/skills/<agent-name>/SKILL.md`)

**Purpose:** Provide the intelligence layer - analysis, synthesis, decision-making.

**What it includes:**
- Agent persona/role from `wiki/agents/*.md`
- Process instructions
- References to KB files
- Output format requirements

**File structure:**
```yaml
---
name: agent-name
description: When to use this skill
allowed-tools: [Read, Write, WebSearch, etc.]
context: fork
model: sonnet
---

# Agent Name

Your role and instructions...

## Process
1. Step 1
2. Step 2

## KB Files
Your KB is at .agents/agent-name/kb/...

## Output Format
...
```

**Example invocation:**
```bash
# User invokes skill
/literature-survey Agriculture & Food Systems

# Or skill is invoked by another agent/system
```

### 3. Execution Modes

#### Mode A: Claude Code (High Quality, Costs Credits)

**When to use:**
- Critical research (literature surveys, credibility checking)
- Complex analysis requiring reasoning
- First-time implementation of new mechanics
- When quality > cost

**How it works:**
1. User/Manager Bot invokes skill: `/literature-survey Agriculture`
2. Claude Code runs skill in forked context
3. Skill uses Rust utilities to read/write KB
4. Results written to wiki + KB files
5. Returns summary to user

**Cost:** Claude Code credits (per token)

#### Mode B: Ollama (Free, Good Enough Quality)

**When to use:**
- Routine surveys (not first-time)
- Batch processing (many surveys overnight)
- When budget constrained
- Good-enough quality acceptable

**How it works:**
1. Manager Bot delegates to Ollama via `game2-core/src/infra/ollama.rs`
2. Ollama gets same instructions as skill (from `wiki/agents/*.md`)
3. Uses same Rust utilities for KB operations
4. Results written to wiki + KB files

**Cost:** Free (local execution)

**Note:** Quality lower than Claude, but sufficient for many tasks.

#### Mode C: Pure Rust (Fast, No Intelligence)

**When to use:**
- Simple heuristics sufficient
- No LLM reasoning needed
- Speed critical
- Fallback when LLM unavailable

**How it works:**
1. Call Rust struct methods directly
2. Simple rule-based logic (if/else, scoring heuristics)
3. Uses same data structures and KB utilities

**Cost:** Free (compiled code)

**Example:**
```rust
// Simple heuristic: get next scheduled domain
let agent = LiteratureSurveyAgent::new();
let next_domain = agent.kb.get_next_domain()?;
```

## Decision Matrix

| Task Type | Execution Mode | Why |
|-----------|---------------|-----|
| First literature survey in domain | Claude | Need high quality research + synthesis |
| Follow-up survey in known domain | Ollama | Good enough quality, save cost |
| Check next scheduled domain | Rust | Simple KB file parsing, no reasoning needed |
| Critical source credibility check | Claude | Need nuanced judgment |
| Routine source logging | Ollama/Rust | Structured data entry |
| Update KB coverage stats | Rust | Pure data manipulation |

## Implementation Pattern

For each agent, implement:

### Step 1: Rust Infrastructure
```rust
// game2-core/src/agents/agent_name.rs

/// Core data structures
pub struct AgentResult { ... }

/// KB utilities
pub struct AgentKB {
    pub fn read_kb(...) -> Result<String, String>
    pub fn write_kb(...) -> Result<(), String>
    // Agent-specific helpers
}

/// Agent struct (delegates to skill or provides fallback)
pub struct Agent {
    pub kb: AgentKB,

    pub fn do_task(&self, params) -> Result<String, String> {
        // Either: delegate to skill/Ollama
        // Or: simple heuristic fallback
    }
}
```

### Step 2: Create Skill
```bash
.claude/skills/agent-name/SKILL.md
```

Include:
- YAML frontmatter (name, description, tools, context)
- Agent persona from `wiki/agents/*.md`
- Instructions for reading KB, doing work, writing results
- Output format requirements

### Step 3: Document in Wiki
Update `wiki/agents/agent-name.md` with:
- Execution modes section
- How to invoke (skill command)
- When to use Claude vs Ollama vs Rust

## Benefits of Hybrid Architecture

1. **Flexible Cost/Quality Tradeoff**
   - Use Claude when quality matters
   - Use Ollama when budget constrained
   - Use Rust when speed matters

2. **Infrastructure Reuse**
   - Same data structures across all modes
   - Same KB utilities
   - Consistent output format

3. **Easy Evolution**
   - Edit skill prompts without recompiling
   - Rust provides stable foundation
   - Can migrate tasks between modes as needs change

4. **Agent Specs Become Prompts**
   - `wiki/agents/*.md` already written like prompts
   - Skills reference these docs
   - Single source of truth for agent behavior

5. **Development Speed**
   - Prototype with Claude skill (fast iteration)
   - Optimize critical paths with Rust later
   - Delegate routine work to Ollama

## Example: Literature Survey Agent

### Rust Infrastructure
- **File:** `game2-core/src/agents/literature_survey_agent.rs`
- **Structs:** `Survey`, `Source`, `MechanicProposal`, `LiteratureSurveyKB`
- **Utilities:** KB read/write, next domain scheduler

### Skill
- **File:** `.claude/skills/literature-survey/SKILL.md`
- **Persona:** From `wiki/agents/literature-survey-agent.md`
- **Tools:** WebSearch, WebFetch, Read, Write
- **Process:** 5-step research process

### Usage

**High-quality research (Claude):**
```
/literature-survey Agriculture & Food Systems
```

**Batch overnight (Ollama):**
```rust
// In manager bot or scheduled task
let ollama = OllamaClient::new();
ollama.run_agent_task(
    "literature-survey-agent",
    "Survey Agriculture & Food Systems domain",
    "gemma3:4b"
)?;
```

**Quick scheduling check (Rust):**
```rust
let agent = LiteratureSurveyAgent::new();
let next_domain = agent.kb.get_next_domain()?;
println!("Next survey: {}", next_domain);
```

## Next Steps

1. ✅ Literature Survey Agent (proof-of-concept)
2. ⏭️ Convert other agents to hybrid pattern:
   - Credibility Analyst (pairs with lit survey)
   - Fun Inspector (playtest analysis)
   - Exploratory Tester (log analysis)
   - PR Agent (code review)
3. ⏭️ Build manager logic for mode selection
4. ⏭️ Add cost tracking across execution modes

## Status by Agent

| Agent | Rust Infra | Skill | Status |
|-------|-----------|-------|--------|
| Literature Survey | ✅ | ✅ | Complete |
| Fun Inspector | ✅ (simple) | ⏭️ | Rust only |
| Gary Agent | ✅ (simple) | ⏭️ | Rust only |
| Simple Playtester | ✅ (simple) | ⏭️ | Rust only |
| Economist Tester | ✅ (simple) | ⏭️ | Rust only |
| Pedantic Engineer | ✅ (simple) | ⏭️ | Rust only |
| Others | ⚠️ (stubs) | ❌ | Pending |

---

See `evidence/agent-implementation-status.md` for detailed agent implementation status.
