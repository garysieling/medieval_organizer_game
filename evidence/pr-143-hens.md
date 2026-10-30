# PR #143 — Hens scratch about the yards

https://github.com/garysieling/game2/pull/143

## What changed
- `LivestockKind::Hen` on the same `Livestock`/`fold` machinery as F2's sheep and goats
- ~4 cottages keep 3 hens (`homes.iter().step_by(5).take(4)` — no RNG consumed, seed stream untouched)
- Hens wander a slow orbit of their door each frame; render via existing `PKind::Chicken`
- Daily egg into house stock (cap 6); a hungry occupant (<40) takes one off the larder for +25 fullness
- Fox nights (`tick % 90 == 75`, 8%): one hen gone, log line, `Problem::Loss` on the household
- F2 roadmap cleanup rode along

## Quality gates
- `cargo test --lib -p game2-core`: 369/369 (4 new `hen_tests`)
- Focused diff: livestock.rs +3, session.rs +117, PLAY.md, roadmap

## Review findings
- Seeding uses no RNG draws → world identical to main; only new entities
- `hen_houses.clone()` per tick avoids an E0502 (self.say needs &mut); trivial cost (4 ents)
- Eggs feed occupants directly rather than via market; kid-errand gather deferred (OddJob plumbing)
