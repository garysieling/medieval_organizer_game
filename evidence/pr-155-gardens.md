# PR #155 — A bed beside the door

https://github.com/garysieling/game2/pull/155

## What changed
- `Game.gardens: Vec<(home, bed_tile, growth)>` — six homes get a soil bed
- `garden_tick` (weekly, `tick%7==5`): +0.2 growth
- `Goal::TendGarden(gi)` — full bed → house stock "marigold"/"feverfew" + `Joy` for occupants
- Herb-wife tends gardens when her shelf <6 raw herbs AND occupants carry no grievances (the trust gate)
- Render: soil strip → green → flowers; `gardens` UiModel field
- Feast log lines joined town-note budget; stale F3 spec deleted

## Quality gates
- `cargo test --lib -p game2-core`: 405/405 (3 new garden_tests)
