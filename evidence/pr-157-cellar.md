# PR #157 — Wheels in the dark

https://github.com/garysieling/game2/pull/157

## What changed
- `BuildingKind::Cellar` — a Home near the fold re-kinded to "the cheese cellar"
- `cheese_tick`: weekly 2 milk from fold stock → `cheeses.push(tick)` (cap 8 wheels); `tick%90==75` the oldest wheel aged >60 days sells at `price × 3 × (1 + age/60)` to the shepherd's purse
- `cellar_trait = "stone-cellar"` — every village's cheese is its own
- Noble-buyer lines joined the town-note budget

## Quality gates
- `cargo test --lib -p game2-core`: 409/409 (2 new cheese_tests)

## Deferred
- Kid "turn the cheeses" errand; `AgingGood` item-level cellar traits
