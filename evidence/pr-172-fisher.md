# PR #172 — The fisher keeps the water's edge

https://github.com/garysieling/game2/pull/172

## What changed
- `Vocation::Fisher` — no inputs, output `"fish"` @3g, "the fish-smoke house" placed at the river's edge (35,18) instead of near-market (swap6 + guaranteed shop)
- `fisher_tick` (weekly): 3 fish most of the year, 1 in the ice-quarter; every 70d "a pale stone pulled from the river — cold, and faintly lit" + "The river keeps its own."
- Test-fragility fix: `finish_meeting` runs 120 sim-days — incidental systems create bonds meanwhile; `player↔corvin` existing no longer proves attendance

## Quality gates
- `cargo test --lib -p game2-core`: 439/439 (2 new fisher_tests)

## Deferred
- SeaMonsterRumor pedlar hook; flood-damage event (storm machinery pending); ice-blocks-fishing modifier
