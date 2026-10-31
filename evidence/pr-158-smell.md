# PR #158 — The day's work leaves its smell

https://github.com/garysieling/game2/pull/158

## What changed
- `villager::Smell { intensity, kind }` component
- `smell_tick` daily: Shepherd→"the fold", Beekeeper→"smoke and wax", Mason→"the cut", Weaver→"the dye", herb-wife→"bitter herbs"; +0.15/day, cap 1.0; `Rain`/`HeavyRain`/`Showers` washes to 0
- `smells_of(v)` — >0.6 → kind
- `match_attempt`: smelly suitor → spark −0.15 + "you smell of the work" bubble

## Quality gates
- `cargo test --lib -p game2-core`: 411/411 (2 new smell_tests)

## Notes
- Kept thin per spec — texture for F20 perfume and later hooks
