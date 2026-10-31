# PR #163 — Something rides over the walls

https://github.com/garysieling/game2/pull/163

## What changed
- `Game.sky: Option<(kind, until)>` — comet `tick%300==150`, eclipse `tick%450==225`, burns 3 days
- `sky_tick`: on trigger, every living villager draws an omen `say` bubble — priest "A sign. Pray.", herb-wife "A gathering-night, that.", Blamed "It burns for me.", Broke "Even the sky's poor.", Love "A star for wishes.", default "Look up."
- UiModel `sky: Option<String>` (Deserialize-safe) → screen-space streak: bright head + fading tail across the top
- Sky lines joined town-note budget

## Quality gates
- `cargo test --lib -p game2-core`: 420/420 (1 new sky_test)

## Deferred
- Priest counsel feeding `fundamentalism`; solstice-anchored festivals
