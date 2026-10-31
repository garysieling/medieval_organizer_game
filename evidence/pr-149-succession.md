# PR #149 — The bench passes on

https://github.com/garysieling/game2/pull/149

## What changed
- `succession_tick` (in `livelihood_tick`): each `Dead` villager's affairs settle once (`Game.succeeded`)
- `inherit_shop` heir chain: senior apprentice (`Craft.master`) → non-kid kin under the same roof → same-vocation journeyman with no workplace
- Transfer: `Craft.workplace` + `Building.worker`; heir gives up their own prior bench first
- Purse: heir gets 4/5, `Problem::DeathDuty` (commons, "the lord's death-duty") marks the lord's fifth
- No heir → `worker = None` → existing `derelict` machinery darkens the shop — the street visibly declines and can recover
- `Problem::DeathDuty` full wiring (tag "the death duty", name, desc, speech)

## Quality gates
- `cargo test --lib -p game2-core`: 388/388 (2 new succession_tests)

## Notes
- `game/succession.rs` (ruler lineages) is a different axis — untouched
- Groundwork for F28 heirlooms and F33 the village continuing
