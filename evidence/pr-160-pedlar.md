# PR #160 — The roving pedlar

https://github.com/garysieling/game2/pull/160

## What changed
- `Pedlar { e, path, selling_until, stall }` — mirrors `Minstrel` (ride `traveller_path` in/out)
- `spawn_pedlar`/`advance_pedlar`: `tick >= next_pedlar` → spawn at road end → walk to market → pitch temp `BuildingKind::TraderStall` with `ribbon`/`needle`/`lute_string` → 3 days → pack up, `destroy_entity` both, `next_pedlar = +60..120d`
- Finery buyers: ≤4 `Want::Finery` villagers `Goal::BuyFrom(stall,"ribbon",1)` — `seller_for` works on the stall naturally
- Gossip: rotating rumor lines ("their harvest failed", "the lord raised the fifth", "a wolf on their common")
- Renders `PKind::Traveller`; visit lines joined the town-note budget

## Quality gates
- `cargo test --lib -p game2-core`: 415/415 (2 new pedlar_tests)
