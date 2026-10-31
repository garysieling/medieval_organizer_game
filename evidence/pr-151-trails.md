# PR #151 — The map learns where people walk

https://github.com/garysieling/game2/pull/151

## What changed
- `Game.trails: HashMap<(i32,i32), Trail>` — wired `sim/trails.rs` (Active Walker Model)
- `advance_travels`: each tile-change calls `step_trail(tr, 1, 0, TRAIL_PARAMS)`; worn tiles scale stride `1 + 0.6·strength`
- `trail_tick` daily: `step_trail(footfall=0)` decay + prune <0.01
- `UiModel.trails` + render: darkened dashes at strength >0.15 — desire-lines visible
- "takes up"/"shuttered" joined the town-note log budget

## Quality gates
- `cargo test --lib -p game2-core`: 392/392 (2 new trail_tests)

## Notes
- Spec's "movement cost ÷ (1+strength)" became stride-speed on walked tiles — same effect without touching `walk_path` determinism
