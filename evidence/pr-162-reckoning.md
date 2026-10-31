# PR #162 — The reckoning that didn't add up

https://github.com/garysieling/game2/pull/162

## What changed
- `Numeracy { Tally, RomanBoard, Positional }` — seeded 60/40 tally/roman
- `Goal::BuyFrom(trader_stall)` on `imported` goods when numeracy ≠ Positional and no translator → 10% friction bite + `reckonings++`
- Every 3rd reckoning → `Problem::ShortChanged` (commons issue, all wiring)
- At 8 reckonings → a Printmaker/Priest becomes `translator` — the dragoman problem solved, friction stops
- Log: "the reckoning didn't add up" / "X keeps the tallies straight now"

## Quality gates
- `cargo test --lib -p game2-core`: 419/419 (2 new reckoning_tests)

## Deferred
- Per-item conversion errors; `AbacusSchool` building
