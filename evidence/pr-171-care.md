# PR #171 — The village walks a little slower for its own

https://github.com/garysieling/game2/pull/171

## What changed
- `villager::Condition { kind }` — "a lame leg" (stride ×0.6), "failing sight", "a mind that needs quiet" on villagers 7/11/15
- `care_tick` (weekly): kin under the same roof first, then deepest bond → visit (bond + Joy), fitting say-line every 3rd week
- Humane by design — a difference is texture and a care relationship, never a punchline

## Quality gates
- `cargo test --lib -p game2-core`: 437/437 (2 new care_tests)

## Deferred
- "Good-for-them" building placement (home-near-market preference)
