# Agent Coordination System Infrastructure

**Category:** Architecture / Infrastructure
**Source:** Backlog item (CRITICAL); implemented across `game2-core/src/infra/` — `worktree.rs`, `task_queue.rs`, `pr_flow.rs`, `agent_kb.rs`.

## Description

The *system* (not an agent) that the agent workforce runs on: isolation primitives so agents can work in parallel without corrupting each other, a durable task queue they pull work from, a PR lifecycle for merging work, and per-agent knowledge bases for learning. Manager Bot and the workforce agents are *users* of this infrastructure; this entry covers the substrate itself.

## Pieces

### Worktrees (`infra/worktree.rs`) — isolation
- `WorktreeManager` wraps `git worktree`. Each agent+task gets `agent/<agent>/<task_id>` branch in `<worktree_root>/<agent>-<task_id>`.
- `detect_conflicts(a, b)` diffs both branches against master and returns overlapping file lists — feeds the Conflict Resolution Agent before merges.
- Invariant: agents never write to `main` directly; all work lands on `agent/*` branches.

### Task queue (`infra/task_queue.rs`) — work distribution
- 9-field line format in `tasks/todo.txt`: id | description | assignee | status | timestamp | deps...
- Dependencies (`task_12` blocks `task_13`) and priorities modeled; Manager Bot reads pending tasks and assigns to agents (see `tasks/manager.log`).

### PR flow (`infra/pr_flow.rs`) — integration
- PR lifecycle + conflict detection between agent branches; pairs with `worktree.rs` conflict detection and the Conflict Resolution Agent's process.

### Agent KBs (`infra/agent_kb.rs`) — memory/learning
- Per-agent knowledge bases at `.agents/<agent>/kb/` (on-disk, e.g. `.agents/implementation-engineer/`, `.agents/manager-bot/`).
- This is the substrate for each spec's "Learning Mechanism": findings, patterns, and retrospectives persist across runs.

## Messaging & locking model

- **Messages:** asynchronous, via artifacts — task queue rows, KB files, PR comments. No direct agent→agent calls; the Manager Bot routes.
- **Locks:** file-level locking is implicit in worktree isolation (two agents can't edit the same checkout). Repo-level conflicts are handled at merge by conflict detection + the Conflict Resolution Agent, not by pre-locking files (keeps parallelism high).

## Gaps to close (next)

- Locking primitive for non-code shared state (e.g. `tasks/todo.txt` itself needs atomic read-modify-write under concurrent agents — today it's single-writer-by-convention).
- Dead-worktree GC + stale branch cleanup policy.
- Backpressure: Manager Bot should cap in-flight tasks per agent.

## Related

- `wiki/ai-agent-workforce.md`, `wiki/agents/manager-bot.md`, `wiki/agents/conflict-resolution-agent.md`
- `wiki/software-design/agent-failure-handling.md`
- `game2-core/src/infra/worktree.rs`, `task_queue.rs`, `pr_flow.rs`, `agent_kb.rs`
