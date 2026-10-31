# PR #147 — The larder keeps its own score

https://github.com/garysieling/game2/pull/147

## What changed
- `spoilage_tick` (weekly, `tick%7==3`): per-good half-life decay on all building stock — bread/milk/eggs weekly, wheat monthly
- `BuildingKind::Granary` wheat takes an extra loss unless `cat_watches` — an occupant's `PetRole::Hunting` cat suppresses vermin
- `Problem::SourGut` for broke+hungry villagers (`tick%9==4`), full reveal/issue wiring
- `spawn_pet` now stores `PetRole` as a component (was baked into mood_effect only — F2's herding check was dead code)
- Mog seeded as `PetRole::Hunting`

## Quality gates
- `cargo test --lib -p game2-core`: 383/383 (3 new vermin_tests)

## Review findings
- PetRole component fix lights up the F2 sheepdog behavior too
- Decay uses aligned week ticks — no rng, no per-frame cost
