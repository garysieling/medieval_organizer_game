# PR #176 — The third defiance is a gibbet

https://github.com/garysieling/game2/pull/176

## What changed
- Summons edict at `debt >= 2` + `lord_esteem > 0.7` + org member → execution: permanent `Dead`, `martyr_grave`, mass `Problem::Grief` ("I watched it and can't unsee it")
- `martyr_tick` (every 30d): org members keep a vigil (Faith satisfy + `record_power`) — "the remembering is a kind of organizing no lord can stamp out"
- Real bug fixed: `repression_debt` was consumed by `mult` before the summons read it — captured as `debt` first
- `Problem::Grief` added (distinct from wolf-specific `Loss`)

## Quality gates
- `cargo test --lib -p game2-core`: 445/445 (2 new gibbet_tests)
