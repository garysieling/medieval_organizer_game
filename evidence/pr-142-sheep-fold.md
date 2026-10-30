# PR #142 — A shepherd keeps the fold

https://github.com/garysieling/game2/pull/142

## What changed
- `Vocation::Shepherd` (guaranteed one per village via a later-duplicate slot overwrite — preserves the RNG stream vs main)
- `BuildingKind::Pasture` fold in the meadow below town; `kind_for(Shepherd)=Pasture`
- `game/livestock.rs`: sheep/goats as entities; flock drifts about a roaming grazing point (tight with a `PetRole::Herding` sheepdog, seeded)
- Milk daily via `work_day` output; wool at shearing day `tick % 90 == 45`
- `Goal::Herd` walks the shepherd out most days; `Problem::Smelly` (-10% reveal) on him
- Wolf nights (`tick % 90 == 60`, 10%): scatter + 0-2 losses + `Problem::Loss` grievance
- `PKind::Sheep`/`Goat` + `draw_beast` blob rendering; fold/treat lines join the town-note budget

## Quality gates
- `cargo test --lib -p game2-core`: 365/365 (6 new flock_tests)
- `evidence/frame-check-50.bmp`: flock grazes south of the wall

## Review findings
- Adding Shepherd to `VOCATION_ROLL` + an extra rng draw reshuffled every adult vocation → 3 regressions (starvation, gravestone, flu-log). Fixed by keeping the table identical and overwriting a later duplicate slot.
- `Goal::Herd` ignored hunger → a starving shepherd oscillated between travel and food-seek for 621 days. Fixed: hungry villagers now abandon travels entirely (general fix), plus a send-time guard.
- Shearing/wolf/"treats" log lines were uncapped "other" entries that evicted the flu lines from the 8-row view → now counted as town business.
