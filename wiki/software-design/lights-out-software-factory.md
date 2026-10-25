# Lights-Out Software Factory

**Category:** Architecture / Meta
**Source:** Backlog item (High) — "Lights-out software factory: fully automated dev pipeline."
**Date:** 2026-10-25

## Description

The fully-automated development factory: agents run continuously, unsupervised, overnight — taking features from spec to merge without Gary in the loop. "Lights-out" = manufacturing term for factories that run with the lights off because no humans are on the floor.

## The factory model

- **Intake:** Gary drops intents into the queue (`infra/task_queue.rs`) — specs, fixes, features
- **Production line:** claim → worktree → implement → test → evidence → PR → review → merge → reload
- **Quality control:** the component-testing framework + determinism + telemetry — evidence, not vibes
- **Escalation:** stuck/blocked/needs-judgment → `needs-gary` queue for morning review
- **Logging:** every run produces a record — what was attempted, what passed, what failed, why

## What lights-out requires (and has)

| Requirement | Where it lives |
|---|---|
| Safe isolation per task | `infra/worktree.rs` |
| Task claiming/dispatch | `infra/task_queue.rs` |
| Model backend | `infra/ollama.rs` (local) |
| Evidence standard | `component-testing-framework.md`, `telemetry-kpi.md` |
| Failure handling | `agent-failure-handling.md` |
| Fix verification | `auto-fix-pipeline.md` |
| No-human-needed gates | contract checks + tests + replay, not human review |

## The real constraint: judgment

- Lights-out works for *mechanical* work — fixes, refactors, spec'd features, tests. It does NOT work for *judgment* — what's fun, what's right, what to build next. The factory produces; Gary decides what the factory should make.
- The honest split: **agents produce volume, Gary produces direction.** A factory that also decides what to build is a different (and worse) product.

## Failure modes to design for

- **Runaway consumption** — an agent loop burning ACUs on a task that can't converge: cap retries, escalate
- **Consensus rot** — agents converging on a wrong-but-passing state: telemetry/invariant checks must be independent of the agent's claims
- **The 3am decision** — something that *needs* a human: the factory halts and files needs-gary, never guesses

## Status

Partially live — `agent_task.rs` + worktree + task_queue + ollama are running; the factory line exists but isn't yet continuous overnight.

## Rating

The name for the whole thing — the thesis of the project as a working system.
