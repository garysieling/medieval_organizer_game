# PR #161 — The still turns flowers to scent

https://github.com/garysieling/game2/pull/161

## What changed
- `Vocation::Perfumer` — inputs `["marigold","lavender"]`, output `"perfume"` @20g, "the perfumery" (swap4)
- `Goal::BuyFrom` on perfume → `Smell { "roses", 0.8 }`
- `smell_tick`: roses persist (not overwritten by work-smell, −0.02/day otherwise)
- Matchmaker smelly-penalty exempts "roses"
- `price("perfume") = 20.0`, `price(marigold/lavender) = 1.0`
- Test robustness: chat-roll padding (n%3 gate), invest `resources` gate

## Quality gates
- `cargo test --lib -p game2-core`: 417/417 (2 new perfume_tests)

## Deferred
- `aqua_vitae`/`glass` inputs (no distillery/glazier chain yet)
