# PR #154 — The year's feast days

https://github.com/garysieling/game2/pull/154

## What changed
- `festival_tick` on `tick % 90 == 45`: four seeded feasts (St. Swithin's, the Harvest Home, Candlemas, St. Martin's)
- Village diverts: all non-role, non-shunned villagers drop travels → `Goal::HeadHome(market_food)`
- Stock comes out: `bread`/`honey` pulled from all building stock → hunger + `Want::GoodFood`/`Joy`/`Music` satisfied en masse
- Bonds: `record_interaction(Conversation, 1.5)` for pairs of co-attendees
- Toasts: first 4 attendees speak a line drawn from their first grievance (the truth comes out in drink)
- Collector: never invited, always shows ("Uninvited. As ever.")

## Quality gates
- `cargo test --lib -p game2-core`: 402/402 (2 new festival_tests)

## Notes
- Completes the opening run of 13 — the first thing that feels like a village, not a sim
- Org-sponsored feasts deferred to F14 (needs the power matrix)
