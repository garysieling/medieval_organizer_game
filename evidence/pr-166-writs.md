# PR #166 — A writ comes down

https://github.com/garysieling/game2/pull/166

## What changed
- `edict_tick` on `tick%120==80`: levy (`grain` off market store), fee (gold off purses), summons (most-grieved villager → `Goal::Exile` "taken for questioning")
- `lord_esteem` response: >0.5 bargained to half; >0.7 + org member → defy → `repression_debt` doubles the next writ
- `Problem::WritBorne` ("the bailiff's writ") lands on villagers each time — a thing to organize about
- Writ lines joined town-note budget

## Quality gates
- `cargo test --lib -p game2-core`: 426/426 (2 new edict_tests)

## Deferred
- Walking bailiff entity; dungeon/ransom (summons uses `Goal::Exile` for now)
