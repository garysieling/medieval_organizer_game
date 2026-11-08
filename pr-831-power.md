## Summary

- **Stale test, not stale code (#831, power):** `test_power_builds_as_its_spent_and_atrophies_idle` still drove idleness through `power_tick`, but #552 deliberately moved atrophy to the daily `influence_tick` (`PowerAtrophySystem`). The test now runs `influence_tick` over the same idle span and also asserts the muscle ends soft (`< 0.1`), proving real decay rather than epsilon drift.
- `power_tick` itself is untouched — it still owns the weekly translator days and the lord's lagging estimate (the sibling test stays green).

#### Test plan
- [x] `cargo test -p game2-core --lib -- session::tests::power` — 2/2 green
- [x] Sim code unchanged; determinism/layering unaffected by construction
- [ ] `cargo test -p game2-core --lib` — remaining red is the other #831 tranches (events, log_priority), fixed in sibling PRs

Part of #831

Generated with [Devin](https://devin.ai)
