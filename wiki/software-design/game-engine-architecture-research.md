# Game Engine / Architecture Research

**Category:** Architecture / Research
**Source:** Backlog item (High) — "Game engine/architecture research."
**Date:** 2026-10-25

## Description

Survey of engine architecture patterns relevant to game2 — what's been tried, what works for sim-heavy games, what to borrow.

## The architecture spectrum

| Pattern | Used by | Fit for us |
|---|---|---|
| **Entity-Component-System** | Unity DOTS, Bevy, flecs | ✓ adopted (`engine/ecs.rs`); sim-first games are the canonical ECS use case |
| **Data-oriented design** | Mike Acton lineage, Unity DOTS | ✓ cache-friendly component layout — matters at scale |
| **Update-loop model** | classic game loop: input→update→render | ✓ base model; our scheduler extends it |
| **Model-View separation** | MVC-style: sim model, render as view | ✓ our architecture — sim is pure, render is a client |
| **Event/command queue** | decoupled input handling, undo, replay | ✓ input stream as data (determinism requirement) |
| **Snapshot + delta** | networked and replayable games | ✓ our checkpoint/history model |

## Key architectural lessons (from industry patterns)

- **Sim is pure, IO is at the edges** — the determinism principle: keep side-effects out of the tick loop (Factorio's determinism work, replay-based games, fighting-game rollback netcode all teach this)
- **Component data as arrays** — the cache-locality argument: struct-of-arrays over array-of-structs for hot component data
- **Systems as explicit functions** — no hidden magic: system = function over matching components, scheduled in declared order
- **The scheduler is the contract** — ordering between systems is a declared dependency graph, not an emergent order

## What the big engines do that we deliberately skip

- Scene graphs, prefab hierarchies, editor-time state, scripting runtimes, reflection — all add engine-specific coupling agents can't navigate
- Real-time physics — we're not a physics game (`2.5D no physics` item completed)
- Live-service/online-first assumptions — we're a single-player sim; no netcode complexity needed

## Reference implementations worth studying

- **Bevy ECS** — closest architectural kin, but heavier than we need
- **flecs** — clean data-oriented ECS design, worth mining for patterns
- **Factorio dev blog** — the gold standard for deterministic sim-at-scale engineering writeups
- **Dwarf Fortress** — not engine-architecture per se but THE precedent for "the sim is the game" design

## Rating

We're on solid ground — the architecture is mainstream-good for this genre; the research mostly confirms rather than redirects.
