# Deterministic Simulation for AI Safety

**Category:** Architecture / Simulation
**Source:** Backlog item (High) — "Deterministic simulation for AI safety & testing."
**Date:** 2026-10-25

## Description

Why the sim must be deterministic: same seed + same inputs → same world, every time. This isn't just a nice-to-have — it's what makes agent-driven development *verifiable*, bugs *reproducible*, and the whole test framework *possible*.

## What determinism buys

| Need | How determinism serves it |
|---|---|
| **Reproducible bugs** | a failing run replays exactly — the bug is a state transition, not a heisenbug |
| **Replay testing** | record a world → regression-test against it → any change in behavior is detected |
| **Agent verification** | "did the fix work?" = replay and compare — mechanical, not judgment |
| **CI/CD** | deterministic runs produce stable test results — no flaky sim tests |
| **Differential testing** | two versions of a system on the same seed reveal behavioral diffs — A/B for the sim |
| **AI safety** | agents can explore consequences offline before touching live state; rollback is exact |

## Requirements for determinism

- **No wall-clock anywhere** — all time from tick count; `Instant::now()` banned in sim code
- **Seeded RNG everywhere** — one central seeded RNG; no `rand::thread_rng`, no hashmap iteration order used for decisions (use `BTreeMap` or sorted iteration)
- **Fixed-point or controlled float** — floats are platform-deterministic but fragile; consider fixed-point for core quantities or pin float ops
- **No external state** — no network, filesystem, or environment reads in the tick loop
- **Ordered scheduling** — systems run in declared order (see `rust-custom-engine.md`)
- **Serialized inputs** — the player input stream is data; replay = re-feed the same input stream

## Implementation in game2

- World state is the only mutable thing; systems are pure-ish functions on it
- The Datomic save/load (`datomic-save-load.md`) gives time-travel for free: replay from any checkpoint
- Telemetry traces are the diffable artifact — a run is characterized by its metric series
- `sim500`/`mega_sim` binaries are the replay/verification harnesses

## The AI-safety angle

Determinism is also the agent-safety story: agents propose changes, the pipeline replays before merging — the game can always be rolled back to a known-good state, and a misbehaving agent's work is a reproducible artifact, not a corrupted world.

## Rating

The invisible backbone — without it, nothing else in the AI-driven architecture works.
