# PR #165 — The circus camps by the gate

https://github.com/garysieling/game2/pull/165

## What changed
- `Circus { e, path, camped_until, camp }` + `spawn_circus`/`advance_circus`/`circus_tick`
- Camps on a `nearest_buildable` tile outside the south wall for 10 days, every ~300-500d
- While camped: villagers <8 tiles get `Joy`/`Music`, −1g/night (light fingers), non-org/non-role → `Problem::Tempted` ("the road's pull", all desires wiring)
- Departure: most-tempted with fewest relationships → `Goal::Exile` + "N has run off with the circus!" — org membership is the counterweight
- Renders `PKind::Traveller` labelled "the circus"; visit lines joined town-note budget

## Quality gates
- `cargo test --lib -p game2-core`: 424/424 (2 new circus_tests)
