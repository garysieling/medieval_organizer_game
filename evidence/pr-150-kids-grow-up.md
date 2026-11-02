# PR #150 — The kids come of age

https://github.com/garysieling/game2/pull/150

## What changed
- `KidGrowth { age_days, errands }` (vocation.rs) seeded alongside `Kid` — staggered ages `(i*180)%700` so graduations trickle
- `RunOddJob` delivery counts the hirer in `KidGrowth.errands`
- `lifecycle_tick` (in livelihood_tick): age_days += 1 daily; at `COMING_OF_AGE` (730) removes `Kid`+`KidGrowth`, adds `Craft` — vocation = top hirer's trade with `master`+`tasked_by` bound, else `roll_vocation`
- Log "N takes up the trade" + toast "N comes of age"

## Quality gates
- `cargo test --lib -p game2-core`: 390/390 (2 new lifecycle_tests)

## Notes
- Pairs with F12: deaths stop being net loss — the roster breathes
- Graduates join the villager loop automatically via `Craft`
