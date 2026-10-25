# Agent Failure Handling & Bad Output Recovery

**Category:** Architecture / Reliability
**Source:** Backlog item (CRITICAL) — "rollback mechanisms, human escalation, fallback strategies, agent quarantine."

## Description

Agents will produce bad output: broken code, hallucinated APIs, tests that pass vacuously, merges that regress. The failure-handling system makes bad output *containable and recoverable* so overnight autonomous runs can't sink the project. Design goal: every failure mode has a mechanical response; escalation to Gary is the *last* resort, reserved for judgment calls.

## Failure taxonomy → response

| Failure | Detection | Response |
|---|---|---|
| Compile/test break in worktree | CI on `agent/*` branch | Auto-revert the offending commit in the worktree; task requeued with failure note in KB |
| Vacuous/gaming output (tests pass, nothing done) | Evidence check — component `evidence()` must show real state change | Task marked "insufficient evidence"; Playtester/Data-Scientist testers verify, not the author |
| Merge regression | Post-merge smoke run of affected components | `git revert` merge → Conflict Resolution Agent owns the repair path |
| Agent loop (repeated identical failing output) | Manager Bot counts attempts per task (attempt budget, e.g. 3) | Task escalated: reassigned to a different agent or human-queued |
| KB poisoning (bad "learning" persisted) | Retrospective Agent reviews KB diffs | KB entry quarantined (moved to `kb/quarantine/`) pending review |
| Cross-boundary writes | Contract check — mod touched undeclared components | Task rejected outright; agent gets contract error, not a merge |

## Mechanisms

- **Rollback:** worktree isolation makes rollback trivial — `git worktree remove` + branch delete discards an entire bad attempt with zero main-branch impact. Merged regressions use `git revert` (append, never rewrite history — consistent with Datomic-style save/load).
- **Quarantine:** an agent whose last N outputs failed validation gets its task scope narrowed (Junior-mode: only well-scoped tasks) until outputs recover. Persistent failure → Manager Bot stops assigning; Retrospective Agent investigates the agent's KB/spec.
- **Fallback strategies:** task graph always has a defined owner; if agent A fails budget, Manager Bot reassigns to agent B with A's attempt notes attached (learning, not just retry).
- **Human escalation:** write to a `needs-gary/` queue file when (a) specs conflict, (b) budget exhausted twice, (c) a change contradicts recorded vision (Gary Agent's domain). Escalation is a *file artifact*, not a silent stall — overnight runs never block on it.

## Invariants

- `main` is never in a state the smoke suite fails — merges gate on it.
- Every auto-rollback leaves an audit trail (task id, agent, evidence, action) in `tasks/` + agent KB.
- No agent may both produce and approve the same change.

## Related

- `wiki/software-design/agent-coordination-infrastructure.md` (worktrees, PR flow)
- `wiki/agents/retrospective-agent.md`, `wiki/agents/conflict-resolution-agent.md`, `wiki/agents/manager-bot.md`
- `wiki/software-design/component-testing-framework.md` (evidence checks)
