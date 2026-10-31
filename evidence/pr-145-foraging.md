# PR #145 — Mushrooms in the shade

https://github.com/garysieling/game2/pull/145

## What changed
- `MushPatch` (tile, poison, next_ready) — ~6 spawn near `map.trees`, 1/3 death-caps
- `Goal::ForageShroom` + `send_shroom`/`send_tile_walk` (shared with `send_forage`, 16-tile cap)
- `mushroom_tick`: hungry (<40) non-role villagers forage every 3d (5d per-soul cooldown); ergot fires ~12% on the year tick — 3 sickened + `Problem::Blamed` on the baker
- Arrive: ID roll = hash(who,tick) < 0.2 + skill*0.7 — good cap +25 hunger, bad read seeds `Disease("bad belly", hidden)`
- `Piece::Shroom` + `shroom_tuft` render + `mush` UiModel field; `Problem::Blamed` wired (the cursed bread issue)

## Quality gates
- `cargo test --lib -p game2-core`: 377/377 (4 new shroom_tests)

## Review findings (fixed in branch)
- Herb-wife stranding: she camped at the last patch — pull radius never covered the meadow → added `Goal::HeadHome`, sent when stocked & far
- Starvation oscillation: hungry check ran after the travelling skip → starving travellers never dropped errands → moved hungry-first in the villager loop
- Role villagers in forage/court pools stranded (no home-pull) → TownRole excluded from both
