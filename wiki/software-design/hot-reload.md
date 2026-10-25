# Hot Reloading System

**Category:** Architecture / Infrastructure
**Source:** Backlog item — implemented in `game2-core/src/infra/hot_reload.rs`.

## Description

Swap a component's implementation without restarting the sim — the enabling feature for overnight AI iteration: an agent fixes a system, reloads just that mod, re-runs the failing scenario, and the world keeps its state.

## Design

- A reload registry maps component name → current implementation handle; `reload(name, new_impl)` swaps the handle behind the component contract (see `modular-architecture-everything-is-a-mod.md`).
- State survives because state lives in the World/ECS, not in the mod's code — reloading swaps *behavior*, not data. This is the real reason the modularity contract matters: components hold no hidden state to migrate.
- Pairs with save/load (`datomic-save-load.md`): checkpoint before reload → if the new impl corrupts state, replay from the checkpoint — reload is safe by construction.

## Failure model

- Reload validates the contract first (provides/requires unchanged); a contract-breaking reload is refused, not applied.
- Post-reload smoke tick: run one scheduler pass, check invariants; auto-rollback on failure (feeds `agent-failure-handling.md`).

## Pipeline

Reload is one stage of the auto-fix loop (see `auto-fix-pipeline.md`): detect → fix → reload → re-verify → commit or rollback.

## Rating

Small module, big unlock — it's what makes autonomous iteration actually cheap.
