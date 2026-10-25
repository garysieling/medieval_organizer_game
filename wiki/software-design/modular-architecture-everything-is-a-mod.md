# 100% Modular Architecture — Everything is a Mod

**Category:** Architecture
**Source:** Backlog item (CRITICAL) — "Swap out different parts of simulation... MAJOR: Need SO MUCH STRUCTURE so AI can run 100% independently overnight."

## Description

Every game system — economy, religion, tech trees, military, UI panels, even core sim loops — is a self-contained, swappable component with a declared interface. Nothing is baked in. A component can be added, removed, replaced, or stubbed without touching unrelated code. This is the enabling architecture for autonomous AI development: an agent can build a component in isolation, test it in isolation, and merge it progressively — because the component contract is the only thing it must satisfy.

This is the central architectural commitment of the project, ahead of any specific system.

## Goals

1. **Independence** — Each component compiles, runs, and is playtested alone.
2. **Substitutability** — Any component can be replaced by a simpler stub, a fancier version, or a competitor implementation behind the same interface.
3. **Progressive merge** — Components are proven in isolation, then merged one at a time into the integrated sim. Integration failures are attributable to a single component boundary.
4. **AI-runnable** — A coding agent given only the component contract can produce a valid component overnight without asking a human anything.

## The Component Contract

Every component must declare:

| Element | Meaning |
|---|---|
| `name` | Unique component id (e.g. `economy`, `religion`) |
| `provides` | Capabilities/events/components it exposes to the world |
| `requires` | Capabilities/data it consumes — everything else is invisible |
| `init(world)` | Registers ECS components + systems on a World |
| `update(dt)` or systems list | Tick behavior, scheduled by the ECS scheduler |
| `evidence()` | Self-check hook — returns a struct describing what the component did this run (events emitted, state changes, invariants) for automated evaluation |
| `completeness()` | "Good enough" self-assessment against declared criteria (see below) |

The registry is a flat list: `registry.register(EconomyMod::new())`. If a mod is absent, the world simply lacks those components/systems — no #ifdef, no special-casing.

## "Completeness / Good Enough" Criteria

Each mod declares what done means so the AI fleet knows when to stop iterating:

- **Compiles + unit tests pass** — baseline
- **Contract conformance** — provides/requires actually satisfied at runtime (registry checks at `init`)
- **Evidence produced** — the component emits verifiable evidence (entities moved, trades executed, attachment styles formed)
- **Invariant hold** — component's own stated invariants hold over N turns (e.g. "money is conserved across trades")
- **No cross-mod writes** — component only touches data it declares in `provides`/`requires` (enforced by review, eventually by type system)

This directly answers the backlog requirement: "define completeness/good enough criteria for bringing in components."

## Phasing

- **Phase 1 (current):** Rust trait `GameMod` + registry; each existing domain (`economics`, `religion`, `tea`, `textiles`...) refactored to satisfy the contract. Components communicate only through ECS components/events.
- **Phase 2:** Manifest-driven loading — mods declared in a TOML/manifest; stubs generated for missing deps; hot-reload (`infra/hot_reload.rs`) swaps a mod implementation without restarting the sim.
- **Phase 3:** True mod loading — dynamic libraries or a scripting/WASM boundary so mods ship independently of the core binary. Only after Phase 1 discipline is proven.

## Implementation Notes

- The existing ECS (`engine/ecs.rs`) is the natural enforcement point: a mod is *only* allowed to register components and systems; the scheduler treats mods as opaque.
- Communication between mods goes through the World (components + event queue), never direct function calls across mod boundaries.
- `evidence()` hooks feed the Independent Component Testing Framework — the playtester runs a mod in a headless world and scores its evidence against the rubric.
- Naming: keep the current flat module names; a "mod" is a packaging/contract concept, not a directory rename.

## Pros

- Overnight autonomous runs are safe — a broken mod can't corrupt unrelated systems
- Natural granularity for the agent fleet: tasks = "implement mod X contract"
- A/B testing of mechanics is free (swap `economy_simple` ↔ `economy_realistic`)
- Gary's "components can be connected and dropped" vision made literal

## Cons

- Upfront contract design is real work; a too-loose contract leaks coupling
- ECS-mediated communication is indirect — debugging a cross-mod interaction requires tracing events, not stack frames
- Full dynamic loading (Phase 3) adds ABI/tooling complexity — defer it

## Related

- `wiki/software-design/component-testing-framework.md`
- `wiki/software-design/hot-reload.md`
- `wiki/software-design/agent-failure-handling.md`
- Backlog: ECS Architecture Confirmed (completed)
