# PR #141 — The herb-wife keeps her hut past the wall

https://github.com/garysieling/game2/pull/141

## What changed
- `TownRole::Herbologist` — exactly one: Willa (villager 30, `HERB_IDX`), Working class, occupation "Herb-wife"
- Her hut: a `Home` building in the meadow outside the west wall, hers alone (excluded from the beds pool)
- 12 wild herb tufts on grass outside the wall; picked tile regrows after a season (90 days); flower spray drawn on the meadow
- She brews hut stock into remedies via `game/herbology.rs` (previously entirely unwired), keeps ~5 on the shelf, forages the nearest ready patch when raw stock < 6
- Sick villagers walk out to her door (`Goal::SeekRemedy`, ≤1 call/15 days); she hands over the best remedy or brews one on the spot for a small fee, else says "Nothing in the hedge for that today"
- Her wander-anchor is the hut, not the market square
- Inspector: "knows the cures · N remedies"; minimap/labels/markers all work via the existing villager path

## Quality gates
- `cargo test --lib -p game2-core`: 359/359 pass
- `cargo run --release --bin game_ui -- --dump-frame 130`: log shows `day 120 Bram has the flu → day 123 Willa treats Bram's flu → day 124 Bram recovered`
- Frames: `evidence/frame-check-30.bmp`, `evidence/frame-check-130.bmp`

## Review findings (self-review)
- `walk_path` refuses occupied destinations — `send()` resolves a passable door tile beside the hut via `at_the_door` (same underlying reason `Goal::BuyAt` to shop tiles never fires; noted, out of scope)
- Keeping the hut in the beds pool shifted every villager's home assignment and put Eliza in a wall-pocket for 38 days (stay_home test) — fixed by excluding it
- The flu test asserts the arrival log line rather than live sickness — a treated Bram may be cured before day +5 (feature working as designed)
- Nit: `arrive` uses `herb_hut.unwrap_or(0)` — benign; `brew_from_stock` fails cleanly to the no-cure line

## Risks
- `SeekRemedy` cap is 30 legs; a far-away sick villager may fail to send and works sick — intended
- Remedies only dent severity for strong infections unless she brews at higher skill over time — intended (skill grows with practice)
