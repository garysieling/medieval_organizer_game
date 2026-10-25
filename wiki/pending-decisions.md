# Pending Decisions Log

Tier 2 decisions made during autonomous runs. Review and override at any time — all are cheap to reverse.

## Active Decisions

### Line-based file format over JSON/serde
- **Date:** 2026-10-25
- **What I chose:** Pipe-delimited text format for task queue (and likely KB files)
- **Alternatives:** JSON+serde, SQLite, binary format
- **Why:** External deps blocked by Windows Application Control; simpler to debug
- **Reversal cost:** Low — swap serializer in task_queue.rs

### Std-only Rust (no external deps)
- **Date:** 2026-10-25
- **What I chose:** std library only for all agent code
- **Alternatives:** serde, chrono, tokio, reqwest, bevy
- **Why:** Windows Application Control blocks build scripts for crates
- **Reversal cost:** Low — add deps to Cargo.toml when environment allows; may want a WSL build instead

### 6-field task format (backwards compatible 5-field)
- **Date:** 2026-10-25
- **What I chose:** `id|desc|assigned|status|created_at|deps` with 5-field fallback
- **Alternatives:** Migrate old file to new format, drop legacy support
- **Why:** Zero data loss, minimal code cost
- **Reversal cost:** Trivial — tighten parser later

### Ollama CLI wrapper over HTTP API client
- **Date:** 2026-10-25
- **What I chose:** Shell out to `ollama` CLI via std::process::Command
- **Alternatives:** reqwest HTTP client, ollama-rs crate
- **Why:** No external deps; CLI verified working. Note: also used HTTP API via curl for proof
- **Reversal cost:** Low — swap internals of OllamaClient later

### Priority-ordered capability matching with fuzzy contains
- **Date:** 2026-10-25
- **What I chose:** Specialists first, impl-engineer last; "testing" matches "*-testing"
- **Alternatives:** Exact match only, capability scoring/ranking
- **Why:** Fixes misrouting without complex scoring
- **Reversal cost:** Low — refine matching rules later

### Agent spec files in `wiki/agents/` per agent
- **Date:** 2026-10-25
- **What I chose:** One file per agent at `wiki/agents/<slug>.md`
- **Alternatives:** Single big workforce doc, KB-resident specs
- **Why:** Matches KB architecture, easier to diff/version
- **Reversal cost:** Low — reorganize later

### Screenshot format: PNG via PowerShell CopyFromScreen
- **Date:** 2026-10-25
- **What I chose:** Full-desktop PNG captures + terminal text captures
- **Alternatives:** Video, windowed capture, ASCII art
- **Why:** Works in this environment; PNG is universal
- **Reversal cost:** Zero — add video later if needed

## Pending Your Review

*(nothing yet — decisions above were all just-made this run)*

## Resolved Decisions

*(none yet)*
