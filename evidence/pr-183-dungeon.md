# PR #183 — The org has a month

https://github.com/garysieling/game2/pull/183

## What changed
- F25's deferred dungeon/ransom: `Game.dungeon: Vec<(Entity, day_taken)>` — a summons *holds* the prisoner, not instant-exile
- `Goal::Exile` arrive: in `dungeon` → "held at the castle"; else → `Dead` as before
- `dungeon_tick` (every 15d): 30-60d + members + org purse ≥20 → ransom (`PlayerCharacter.resources −20`) → `Goal::HeadHome` + "walks home"; 60d unanswered → `Dead` + "hanged at the castle"
- Completes F25's "stocks/dungeon for the ringleader, who must be ransomed or rescued"

## Quality gates
- `cargo test --lib -p game2-core`: 455/455 (3 new dungeon_tests)
