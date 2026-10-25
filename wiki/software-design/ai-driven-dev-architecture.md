# AI-Driven Development Architecture

**Category:** Architecture / Meta
**Source:** Backlog item (High) — "AI-driven development: agents building the game."
**Date:** 2026-10-25

## Description

The meta-architecture: how the project is *built by AI agents* — the workflow, the safety rails, the org chart of agent roles. This is the umbrella entry; details live in `agent-coordination-infrastructure.md`, `component-testing-framework.md`, `agent-failure-handling.md`, `modular-architecture-everything-is-a-mod.md`, `ollama-integration.md`, `lights-out-software-factory.md`.

## The thesis

Gary doesn't write the game; Gary *manages a software organization* that writes the game. The project's product is twofold: the game AND the agent system that builds it — `agent_task.rs` and the fleet already execute real tasks.

## The org structure

- **Gary = GM/vision-holder** — sets direction, defines evidence standards, resolves agent escalations ("needs-gary"), reviews agent output
- **Specialist agents** — PM (priorities), Architects (design/code), Developers (implement), Testers (playtest + Data Scientist variants), Aesthetic Consultant, LLM Engineer, Writer, Community Manager, QA
- **Gary Agent** — vision alignment checking (`wiki/agents/gary-agent.md`)

## The workflow

```
intent (Gary) → tasks (task_queue rows) → agent claims → worktree →
implement → evidence (telemetry/diff/tests) → PR → review → merge → hot reload
```

## The safety architecture (the important part)

| Layer | Mechanism |
|---|---|
| **Isolation** | each task runs in a git worktree (`infra/worktree.rs`) — no agent clobbers another |
| **Contract** | every mod declares provides/requires (`modular-architecture-everything-is-a-mod.md`) — agents can't silently change interfaces |
| **Evidence** | no claim without verification: telemetry runs, test results, diffs — the evidence standard from `component-testing-framework.md` |
| **Verification** | the framework, not the agent, decides "did it work" — agents propose, the pipeline disposes |
| **Rollback/quarantine** | bad work is reverted, quarantined, or escalated (`agent-failure-handling.md`) |
| **Determinism** | the sim is replayable — "same seed, same world" makes agent-produced bugs reproducible (`deterministic-simulation-ai-safety.md`) |

## The real insight

The codebase is *designed for agents*: module boundaries are contracts, tasks are rows in a queue, evidence is mechanical, state is append-only and replayable. A human-codebase-shaped project would defeat agent development — this one is shaped for it.

## Rating

This is the product under the product — arguably more novel than the game.
