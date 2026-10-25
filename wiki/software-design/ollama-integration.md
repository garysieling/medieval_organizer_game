# Ollama Integration for Test Automation & Bug Fixes

**Category:** Architecture / AI Infrastructure
**Source:** Backlog items — "Local AI Model Integration for Bug Fixes" + "Ollama Integration for Test Automation — slow overnight test runs vs fast feedback loops." Implemented: `game2-core/src/infra/ollama.rs` (OllamaClient, gemma3:4b verified), `src/bin/agent_task.rs`.

## Description

Local LLMs via Ollama as the agent fleet's brain — no API keys, runs overnight on the dev machine, cheap enough to run dumb tasks at scale. `agent_task.rs` already executes real tasks through the client; `infra/ollama.rs` is the integration point every agent spec's "Claude Skills/Rules" concept maps onto locally.

## Two loops (the design distinction from the backlog)

| Loop | Speed | Model role | Use |
|---|---|---|---|
| **Fast feedback** | seconds-minutes | small local model (gemma3:4b class) | lint-level checks, spec-conformance review, triage of test failures, draft fix attempts |
| **Slow overnight** | hours | local model fleet, possibly larger pulled models | full playtest scoring, exploratory data-science passes, refactors, lit-review processing |

The split matters: a model slow enough to be thoughtful can't be in the dev loop, but is fine unattended overnight — so the architecture routes by urgency, not capability.

## Task-automation architecture

- `task_queue.rs` rows → an agent's prompt template (spec file + task row + relevant KB) → `OllamaClient` → result → evidence check (telemetry/diff) → PR or failure path (`agent-failure-handling.md`).
- Model swap is a config value — upgrade the fleet by changing a string when a better local model lands; nothing else changes. (The LM Studio install on this machine is an alternate local backend for bigger models.)
- Everything the model emits is treated as *untrusted draft* — the component-testing framework verifies, never the model's own claim (vibe-coded trust is the classic failure here).

## Bug-fixes specifically

- The "local AI model integration for bug fixes" loop: failing test/trace → model proposes fix on `agent/*` worktree → fast-loop checks (build+tests) → human/agent review via PR flow. The model *assists*, the pipeline *decides*.

## Rating

Already running (gemma3:4b verified) — the entry documents the routing architecture, not a greenfield build.
