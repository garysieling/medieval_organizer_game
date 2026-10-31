# PR #174 — The work goes on

https://github.com/garysieling/game2/pull/174

## What changed
- `player_passed` + `continuation_tick`: on player `Dead`, successor = living org member with deepest `Relationship` to the player (org-first — the organizing's institutional), else kin under the same roof
- `PlayerCharacter` transfers (resources + standing, influence halved — "theirs to make"); `self.player` swaps; "the work goes on — N takes up the organizing"
- No successor → "the work dies with them" — the truest end
- Heard ledger is institutional (lives on org members) — no transfer needed

## Quality gates
- `cargo test --lib -p game2-core`: 442/442 (2 new continuation_tests)
