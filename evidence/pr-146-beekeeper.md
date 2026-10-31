# PR #146 — A beekeeper keeps the skeps

https://github.com/garysieling/game2/pull/146

## What changed
- `game/apiary.rs`: `HiveKind{Skep,Log}`, `Hive`, `spawn_hive`, `hives_of`
- `Vocation::Beekeeper` (honey output, "the apiary") — second RNG-free slot overwrite (rposition, non-farmer duplicate)
- `BuildingKind::Apiary` — seeded east of the fold, re-kinded like the fold
- `apiary_tick`: `tick%90==50` honey×2+wax/hive into apiary stock; wax→`candle` into church; `tick%90==20` church burns 1 candle → empty → priest `Problem::NoInput("wax")` + "the church is dark"; `tick%90==70` 8% swarm follows a villager a day
- `seal_aid_pact` consumes 1 wax when a seller stocks it (tests unaffected — no wax economy → still seals)
- `Piece::Skep`/`Piece::Swarm` + `draw_skep`/`draw_swarm` + `skeps`/`swarm` UiModel fields

## Quality gates
- `cargo test --lib -p game2-core`: 380/380 (3 new apiary_tests)

## Review findings (fixed in branch)
- Eliza strayed 38d at a west patch (Idle, fed): the sim never sends idle far-from-home villagers home → added `Goal::HeadHome` sends for villagers >6 from home AND post
- Death-cap test: spread hash sampling + forced low skill
- skep/swarm log lines joined the town-note budget
- Skep-harvest ×2/×1 tension deferred (flat yield v1)
