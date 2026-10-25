# Independent Component Testing / Play Testing Framework

**Category:** Architecture / Testing
**Source:** Backlog items (CRITICAL + High) — "each game component can be independently tested and play tested. AI runs overnight testing individual components. Progressive merging strategy."

## Description

A test harness that spins up a headless `World` containing exactly one component (plus declared stubs for its `requires`), runs it for N turns, and scores the emitted evidence. Components are proven in isolation AND re-proven after each progressive merge step, so "works alone" and "works together" are both checked continuously.

Serves two audiences:
- **Correctness** — unit/integration tests per component (traditional)
- **Playability** — agent playtesters drive a mod's mechanics through its API and score against the Fun Rubric — this is the EA/Sims Mobile pattern: recreate/drive mechanics as a separate system, don't drive the client (see `wiki/literature-reviews/sprint-1-game-design-simulation.md`, source 4).

## Architecture

```
component_test/
├── harness.rs        # World builder: load mod + stubs, run N ticks
├── stubs/            # minimal fake providers for unmet `requires`
├── evidence.rs       # collects component evidence() output + telemetry
├── rubric.rs         # scores evidence vs component's declared criteria
└── reports/          # per-component JSON/markdown run reports
```

### Run flow
1. Read the mod's contract (`provides`/`requires`).
2. Build a `World`; register the mod + auto-generated stubs for unmet requirements.
3. Tick the scheduler for the scenario's turn count (from the mod's test manifest).
4. Collect: component `evidence()`, telemetry metrics/spans, invariant checks.
5. Score against the rubric → pass/fail + narrative (what happened, why it's wrong).
6. Emit a report an agent (or Gary) can read.

### Progressive merge
- Stage 0: each mod alone → must pass alone.
- Stage 1: pairwise with declared collaborators (economy↔market, religion↔community).
- Stage 2: full registry (`mega_sim` binary already exercises this pattern).
- A merge step only runs if all previous stages pass → failures are attributable to the newest boundary crossed.

## Agent Integration (overnight mode)

- Each mod gets a `test manifest`: turn count, seeded RNG, invariants, rubric weights.
- The Exploratory Tester (Data Scientist) sweeps manifests; the Simple Playtester executes goal scripts ("build 5 units", "form a congregation"); the Fun Inspector scores traces.
- Failures file to the agent's KB (`agent_kb.rs`) and, if configured, spawn a fix task via `task_queue.rs`.

## Completeness criteria (component exits testing)

- Deterministic: same seed → same evidence digest (allowing approximate determinism per the 2.5D decision)
- Contract clean: no undeclared reads/writes detected
- Evidence sufficient: rubric score ≥ component's declared threshold
- Merge-safe: passes pairwise stage with all declared collaborators

## Pros

- Failures localize to one component — exactly what's needed for unattended AI iteration
- Produces the "evidence = software output" trail Gary expects (traces, metrics, not screenshots)
- Rubric-scored playtests turn "is it fun?" into a checkable artifact

## Cons

- Stub fidelity: a stub that's too simple can hide integration assumptions (mitigate: stage-1 pairwise tests early)
- Rubric tuning is judgment work — expect false alarms while rubrics mature

## Related

- `wiki/software-design/modular-architecture-everything-is-a-mod.md`
- `wiki/fun-rubric.md`, `wiki/agents/simple-playtester.md`, `wiki/agents/exploratory-tester-data-scientist.md`
- `game2-core/src/bin/mega_sim.rs` — existing multi-system run this formalizes
