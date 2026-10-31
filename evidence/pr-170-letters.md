# PR #170 — Put it in a letter

https://github.com/garysieling/game2/pull/170

## What changed
- `letters: Vec<(from, to, due)>` + `Letter` struct + `literate(v)` (priest/printmaker/translator/player)
- `letter_tick`: due letters arrive (bond `record_interaction` + `Joy` + "a letter comes"); literate writers send one monthly to their strongest bond, ≤3 in transit
- Illiterate recipients pay the priest 1g for the reading — "The Father read it out - word from home."

## Quality gates
- `cargo test --lib -p game2-core`: 435/435 (2 new letter_tests)

## Deferred
- Contract-breach evidence letters; explicit pedlar-post tagging
