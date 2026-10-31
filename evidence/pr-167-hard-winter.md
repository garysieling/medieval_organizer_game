# PR #167 — Every other winter runs long

https://github.com/garysieling/game2/pull/167

## What changed
- `Game.harsh_winter` on `tick%730 ∈ [400,545]` — every other winter's a hard one
- While harsh: `Flurries` persist, market grain −1/5d, villager hunger −8/10d, `Problem::LongWinter` commons ("the long winter", all desires wiring)
- The answer rides `lord_esteem`: >0.5 → org shares out (Joy + hunger + `record_power` + "the village pulls together"); ≤0.5 → `Blamed` on the herb-wife first, then heart-helper, then collector — the witch-trial path in a cold year
- Winter lines joined town-note budget

## Quality gates
- `cargo test --lib -p game2-core`: 429/429 (3 new winter_tests)

## Notes
- Overlap checked: `bin/hard_winter.rs` is the raw-ECS standalone (already merged); this is the in-game scenario
- Deferred: wolf-pressure, crop-stunting
