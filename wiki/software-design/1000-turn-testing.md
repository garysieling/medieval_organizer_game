# 1000+ Turn Automated Testing Mode

**Category:** Design / Testing
**Source:** Backlog item (High) — "1000+ turn automated testing: long-running sim validation."
**Date:** 2026-10-25

## Description

Long-horizon automated runs: let the sim run for 1000+ turns unattended and check that the world stays coherent, doesn't drift into degenerate states, and produces the long-run dynamics the design intends. `sim500`/`mega_sim` binaries are the harnesses.

## Why long-horizon specifically

- Short runs hide the bugs that matter — resource leaks, degenerate equilibria, population crashes/booms, runaway systems only show up *over time*
- The game's own claim is "a world that lives" — 1000 turns is where that claim is tested
- Emergence needs room — a system's interesting behavior may take hundreds of turns to appear

## What to check in a long run

| Check | What breaks it |
|---|---|
| **Population stability** | extinction, explosive growth, demographic collapse |
| **Economic coherence** | runaway inflation, universal starvation, dead markets |
| **Psych/social health** | everyone stressed, total social fragmentation, universal gloom |
| **Diversity** | monoculture — one crop, one personality, one outcome; a dead world |
| **Performance** | tick time creep, memory growth, entity explosion |
| **Invariant violations** | negative quantities, orphaned entities, broken contracts |
| **Emergent stories** | the run produced *events* — not a flat line of stable ticks |

## The harness design

- Deterministic seed → the run is replayable (`deterministic-simulation-ai-safety.md`)
- Periodic checkpoints (`datomic-save-load.md`) — inspect any point, restart from there
- Telemetry export every N ticks — the run's *trace* is the diagnostic artifact
- Multiple seeds in parallel — a long run is a distribution over outcomes, not a single path
- Agent-driven: the fleet runs these overnight; failures enter the queue

## The emergent-health rubric

A good 1000-turn run shows: variation over time, historical texture (eras, rises, falls), multiple storylines, no degenerate equilibrium. A run that just *settles* is a design failure — the world should stay interesting.

## Related

- `ai-player-beta-testing.md`, `fun-rubric-automated-testing.md`, `deterministic-simulation-ai-safety.md`, `telemetry-kpi.md`, `emergent-storytelling.md`

## Rating

The stress test — where the sim's claims get proven or disproven.
