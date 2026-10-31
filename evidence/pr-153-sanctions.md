# PR #153 — The ladder of shame

https://github.com/garysieling/game2/pull/153

## What changed
- `game/sanctions.rs`: `Sanction` ladder Gossip→Shun→Boycott→Picket→Stocks→Exile, `rung_for(grievance_count)`, `label`, `blame_worthy`
- `sanction_tick` (weekly): most-grieved villager (≥3 active grievances) climbs one rung; `sanctioned: Vec<(Entity, Sanction, until)>` lapses at +60d
- `apply_sanction` per rung: Gossip log+say, Shun (meetings+courtship exclusion via `shunned()`), Boycott (`boycotted()` filters `seller_for` at all 3 call sites), Picket (crowd-at-door log+say), Stocks (`reputation_qsp` crowd reaction: rep>0.5 → backfire `GreatDeed`+"the crowd would have none of it", else `Betrayal`+stands at the store), Exile (`Goal::Exile` → edge tile → `Dead` on arrival)
- `ReputationState` seeded lazily (score 0.45, `rep_of`/`rep_apply` helpers)

## Quality gates
- `cargo test --lib -p game2-core`: 400/400 (4 new sanction_tests)

## Notes
- The emergent witch-trial path: ergot `Blamed` and suspected cures escalate through the ladder without scripting
- Exile arrival uses the `Dead` marker — gone is gone
