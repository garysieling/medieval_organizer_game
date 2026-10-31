# PR #169 — A thing made to outlast its maker

https://github.com/garysieling/game2/pull/169

## What changed
- `villager::Heirloom { maker, owner, name, born_tick, stories }` on item entities
- `heirloom_tick` (every 90d): the surest hand (skill >0.7, luthier/glazier/toolmaker/weaver/tailor) lays down a masterwork — "a lute", "a window", "a fine toolset", "a woven hanging" — once a lifetime per maker
- Bequest: on owner death, passes to a living kin under the same roof ("a bequest picks kin"), + story "passed from the dead" + say line "It's mine now. I'd rather have them back."

## Quality gates
- `cargo test --lib -p game2-core`: 433/433 (2 new heirloom_tests)

## Deferred
- Inspector lineage line ("made by old Berta, played at the winter feast"); feast-time story tags; sold/lost paths
