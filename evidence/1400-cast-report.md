# Cast report baseline — E0-1 (#1400)

`cargo run -p game2-core --bin cast_report -- 100 both` on
`main` after E1-1..E2-4 (seeded personas, ghosts and caps landed).

## Fixed cast (authored, 100 seeds)

- Fewest distinct trait sets: **26 in 31 villagers** (seed 0)
- Biggest hook pile-up: **9 holders** (seed 0) — the soldier-brother
  pile-up the doc cites, reproduced verbatim by the old index hash
  (`hooks::seed`).

## Seeded cast (100 seeds x 31 villagers)

| check | target | seeds failing |
| --- | --- | --- |
| shared signatures | 0 | 100 |
| most holders of one wound | <= 2 | 0 |
| archetypes present / biggest share | >= 6 / <= 20% | 16 |
| nearest pair distance | >= 0.50 | 99 |
| contradictions per cast | ~1 in 6 +/-10% | 95 |

- nearest pair: min 0.211, mean 0.374, max 0.524 (worst seed 84, slots 23 and 28)
- trait sets: fewest 17 in 31, mean 21.7 (identical-line rate ~30%)
- fixed cast identical-line rate: 16% (5 of 31 share a trait set)
- contradictions per cast: mean 0.7 (target ~5)
- distinct wounds per cast: mean 5.5
- archetype holders per cast: mean 2.5-3.0 each, some absent in a
  seed (min 0), worst single-cast share 9/31

## Findings for E1-5 (#1413)

- Wound pile-ups: **already fixed** by E2-3/E2-4 (0 failing seeds at
  the <=2 cap). The validator only needs to keep watching it.
- Signatures: the pool is 33 landmarks for 31 villagers with
  independent draws - collisions are near-certain and a per-villager
  reroll cannot reliably clear them. Dealing without replacement
  (shuffle the pool, deal 31) is the fix, not rerolling.
- Nearest pair: floor 0.50 is unreachable as drawn (max observed
  0.524, mean 0.374). Either the floor tunes down to ~0.30 or the
  reroll rejects pairs under a real floor.
- Contradictions: mean 0.7/cast vs ~1-in-6 target - the contradiction
  draw is much rarer than the design wants; check the draw band.
- Archetype spread: 16% of seeds exceed the 20% share cap; reroll of
  passes 3-6 does not move archetype, so the check stays advisory or
  the archetype must join the rerolled passes.
