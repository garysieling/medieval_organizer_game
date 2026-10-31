# PR #182 — The pedlar swears he saw it glow

https://github.com/garysieling/game2/pull/182

## What changed
- F31's deferred SeaMonsterRumor: `Pedlar.rumor_sold` — once per visit while selling, he tells the river tale ("a thing glows under the river — a pale stone, older than the walls") and a hearer carries it on
- The pale stone's already in the world (F31's river-strange finds); whether the tale's true is a world parameter

## Quality gates
- `cargo test --lib -p game2-core`: 452/452
