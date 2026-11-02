# PR #144 — The heart-helper keeps an eye on lonely hearts

https://github.com/garysieling/game2/pull/144

## What changed
- `TownRole::Matchmaker` seeded on one roster elder (`villagers[27]`, reads Working class, "Heart-helper" in the inspector)
- `matchmaker_tick`: every 4 days a `lonely_hearts` villager (grown, unmet `Want::Love`, no `Lover`) is sent `Goal::TalkTo(her)` — she holds court, hearts come to her
- `match_attempt`: spark roll = 0.5 + 0.1/shared want - 0.15 if `Rival`-related; `Family`/`Lover`/30-day-tried pairs excluded; deterministic (a,b,month) hash — no rng spent
- Success → two `RelKind::Lover` rels + `desires::satisfy(Love)` for a year + 2g fee + log ("a match the families won't like" if feuded); miss → 30-day cooldown
- `CtxAct::Matchmake` when the heard ledger knows ≥2 "loneliness" hearts — the player brokers the same roll
- match/fox/fold/"treats" lines joined the town-note budget

## Quality gates
- `cargo test --lib -p game2-core`: 373/373 (4 new `heart_tests`)

## Review findings
- First draft had her WALKING to hearts → chained trips kept her away from home (stay_home fail). Now hearts come to her.
- `match_attempt` used `rng.next()` initially → moved to a deterministic hash so the seed stream is untouched.
- flu-log eviction recurred via new log lines → same is_town_note fix.
