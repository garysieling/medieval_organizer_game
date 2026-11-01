# PR #418 — Perf: bound the long game

PR: https://github.com/garysieling/game2/pull/418
Branch: `perf/mem-cpu-audit` (worktree `../game2-perf`)
Date: 2026-11-01

## Method

New probe: `cargo run --release --bin perf_probe [days] [seed]` — prints
per-day wall time, log lines, villager/entity/relationship counts as CSV.
`scripts/sample-mem.ps1` polls RSS/CPU of a running probe.

Baseline: `perf_probe 800 42` headless, speed 4, idle driver.

## Baseline findings (before)

- Day cost: 4.9ms at day 1 -> ~19ms at day 800 (~3x growth, ~7.9s total)
- Relationships: 20 -> 1373; entities: 137 -> 1661; villagers 31 -> 58
- RSS: 6.5 -> 7.1MB over the run (+0.6MB, steady, unbounded)
- `find_relationship` = query()+sort+scan of all bonds, 72 call sites
- separation pass = 2 hashmap lookups x v^2 pairs, every frame
- `ui_model()` cloned whole TileMap + whole log every frame
- `game.log` never capped; view re-scanned it via `visible_log` per frame
- correctness: levy `villagers.remove(idx)` left `was_sick`/`died_on`
  misaligned for everyone after the removed kid

## Fixes

- `World`: `iter`/`find` (no alloc, no sort) + type-keyed resources
- `RelIndex` pair index; `add_relationship` registers bonds; fallback
  scan preserves lowest-id-wins (determinism, verified by the flu test)
- `bonds_of`/`bond_map` (creation-ordered) for the per-villager bond scans
- separation pass: per-frame Pos/yield snapshots; fixture pairs skipped
- `UiModel.log` = visible lines only; view iterates them directly
- `game.log` capped: trim to 12288 when over 16384
- `was_sick`/`died_on` removed in step with the roster

## After

- `perf_probe 800 42`: 3.8s total (2.1x); day-800 cost ~7ms vs ~19ms
- `balance_test 150 42`: 96/100, no flags
- `cargo test --release`: all suites green (668 lib tests + integration)

## Follow-ups noted (not done)

- TileMap still cloned per frame (constant cost, bigger refactor)
- Dead villagers keep all components; graves need only Pos+marker
- `World::query()` still sorts everywhere by design
