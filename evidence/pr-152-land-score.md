# PR #152 — The land keeps score

https://github.com/garysieling/game2/pull/152

## What changed
- `sim/environment.rs`: `LandScore { tree_cover, pit_water, effluent }`, `wood_yield`/`stone_yield`
- `Goal::Gather(good, tile)` — gathers carry their tile so the land knows where feet went
- Arrive: wood −0.12 cover on that tile (yield 2→1 at <0.3); stone +0.03 pit_water (yield 2→1→0 at 0.7/1.0); "the pit's drowned" log at flood
- `land_tick`: canopy regrowth +0.005/day on undisturbed/unbuilt only; effluent +0.004/3d on river tiles below the south wall (upstream stays clean)
- Thresholds → `Problem::ThinWood` (toolmaker/luthier), `DryPit` (mason/toolmaker), `StinkingReach` (homes near foul tiles) — all unscoped commons
- Render: stumps, murk tint, pit-pond; seeded head-start (pit 0.5, cover 0.85, reach 0.15)

## Quality gates
- `cargo test --lib -p game2-core`: 396/396 (4 new land_tests)

## Notes
- The land's grievances are the bridge issues F43 made matter — organizing content, not scenery
- Gravestone test tolerates a passer-by's edge over the stone pixel
