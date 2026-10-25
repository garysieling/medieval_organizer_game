# Datomic-Style Save/Load with History & Replay

**Category:** Architecture / Persistence
**Source:** Backlog item (CRITICAL); implemented in `game2-core/src/infra/save_load.rs`, exercised by `src/bin/save_load_demo.rs`.

## Description

Game state is treated as an accumulating history, not a mutable blob to overwrite — the Datomic model. Each checkpoint captures world state at a tick; replay jumps back to a prior point and re-simulates forward. This gives time-travel debugging, deterministic replay for tests, and an audit trail of how the simulation got to its current state.

## Design

- `Checkpoint { tick, id, data }` — snapshot of world state at a tick (currently entity/component digest; becomes full component serialization in Phase 2).
- `History { checkpoints, current_tick }` — append-only list. `checkpoint(world, label)` appends; `replay_from(id)` retrieves the point to resume from; `latest()` for quick resume.
- Demo: `save_load_demo.rs` checkpoints → mutates → replays → verifies.

## Why Datomic-style (not snapshot files)

| Property | Snapshot files | Datomic history |
|---|---|---|
| Debugging | "state was wrong at save" | diff any two ticks, find the bad turn |
| Testing | each test needs fixtures | replay from the checkpoint *before* the interesting event |
| Autonomous agents | opaque binary | agent can bisect history to find where a mechanic broke |
| Performance cost | rewrite whole file | append-only; checkpoint cadence tunable |

## Phasing & performance plan

- **Phase 1 (now):** in-memory `Vec<Checkpoint>`; fine for demos/short sims.
- **Phase 2:** full component serialization (bincode/serde per component type), disk-backed checkpoints, interval policy (every N ticks + on-event).
- **Phase 3 (when it hurts):** structural sharing — store diffs between checkpoints rather than full snapshots; index by component for partial replays. *Planned compromise:* if deep history is too slow, cap retention (keep last K + sparse milestones) — correctness of replay is non-negotiable, retention depth is the dial.

## Pros

- Time-travel debugging and bisect-by-replay for the AI fleet
- Save points become free (any checkpoint is a save)
- Aligns with matrix-multiplication turn model: history = recorded products; replay = reapply from a basis state

## Cons

- Memory growth unbounded in Phase 1 (acceptable: runs are short)
- Full serialization needs stable component schemas — coordinate with the modularity contract (versioned `provides`)

## Related

- `wiki/software-design/modular-architecture-everything-is-a-mod.md`
- Backlog: "Turn-Based Matrix Multiplication Model" (completed)
- `game2-core/src/infra/save_load.rs`, `src/bin/save_load_demo.rs`
