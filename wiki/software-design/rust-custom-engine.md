# Rust Custom Engine Planning

**Category:** Architecture / Engine
**Source:** Backlog item (High) — "Rust custom engine planning: ECS architecture, rendering, audio."
**Date:** 2026-10-25

## Description

Why a custom engine in Rust, and how it's organized: the game is simulation-first (a society model that could run headless for 1000 turns), so the engine is a thin presentation layer over a deterministic ECS core.

## Why custom (not Unity/Godot/Bevy)

- **Headless-first requirement** — sim runs without rendering (AI playtesting, 1000-turn runs, agent fleets); Unity/Godot couple sim to scene/render
- **Determinism requirement** — lock-step replay/verification requires total determinism: custom sim controls every nondeterminism source (`deterministic-simulation-ai-safety.md`)
- **Agent-legibility** — the codebase must be navigable by AI agents: plain Rust modules, no engine-specific scene graphs or editor-generated code
- **The fun is the sim** — most value is in the society model, not graphics tech; a custom renderer for isometric 2D tiles is small enough to own
- **Bevy was too heavy** — full ECS framework carries entity/component machinery we don't need; our ECS is simpler and more legible

## Architecture

```
game2-core/
  engine/    — ECS (World, entities, components, scheduler), render loop, input, audio
  infra/     — task_queue, agent_kb, worktree, ollama, telemetry, save_load, hot_reload, pr_flow
  sim/       — domain systems (psychology, economics, illness, religion, community, climate, magic, emergence...)
  game/      — game-content systems (economy, factions, contracts, tech_tree, recipes, military, lifecycle...)
  ui/        — isometric, minimap, character_sheet, dialogs, graph, tech_view
  agents/    — agent binaries/specs
```

## The ECS (our version)

- `engine/ecs.rs` — entities as indices into archetype/sparse component stores; systems registered on a scheduler with explicit ordering; World owns all state
- Deliberately *simpler* than Bevy ECS: fewer abstractions, more legible to agents, faster to onboard
- Scheduler ordering is the determinism anchor — systems run in declared order, no hidden parallelism

## Rendering

- Custom 2D isometric tile renderer (`ui/isometric.rs`) — software/pixel rasterization; the Caesar-2 look is cheap enough to draw directly
- No GPU dependency required for the sim; render is a viewer, not the game

## Audio

- Ambient/sfx layer (`sfx-ambient.md`) — decoupled from sim; events raise audio cues

## Principles

- **Sim is pure, render is a client** — the game could render to BMP, terminal, or browser without sim changes
- **No engine magic** — no reflection, no macros-generated code, no hidden update loops; everything is a function on World
- **Modularity** — every sim/game system is a mod (`modular-architecture-everything-is-a-mod.md`)

## Rating

Correct call for this project — the custom engine is what makes the whole AI-driven/society-sim thesis executable.
