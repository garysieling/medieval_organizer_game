# PR #177 — A village already mid-story

https://github.com/garysieling/game2/pull/177

## What changed
- `Scenario { FeverYear, CircusWinter, WidowsShop }` + `Game::new_scenario`
- FeverYear: `Disease` "the fever" on 3 villagers (Moderate, contagious)
- CircusWinter: `tick=400` (harsh window) + `spawn_circus`
- WidowsShop: shop worker `Dead` + `succession_tick` → derelict + "who's to keep it now?"
- Note: `save_load.rs` is a checkpoint skeleton — presets are the honest read

## Quality gates
- `cargo test --lib -p game2-core`: 448/448 (3 new scenario_tests)
