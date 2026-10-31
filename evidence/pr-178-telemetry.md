# PR #178 — The village reads its own innards

https://github.com/garysieling/game2/pull/178

## What changed
- `Game::telemetry() -> Vec<(&str, f64)>` — 15 gauges: villagers.alive/hungry, org.members/power/lord_esteem, sanctioned, trails.heat/max, stock.grain/bread, env.tree_cover, price.grain/wood/fish, winter.harsh
- `telemetry_lines()` + `UiModel.debug_lines` — the tuning panel's data, exposed for the overlay
- Distinct from `infra/telemetry.rs` (the KPI recorder) — these are sim gauges, not metrics spans

## Quality gates
- `cargo test --lib -p game2-core`: 449/449 (1 new telemetry_test)
