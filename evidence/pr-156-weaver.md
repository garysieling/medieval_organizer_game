# PR #156 — The loom turns wool to cloth

https://github.com/garysieling/game2/pull/156

## What changed
- `Vocation::Weaver` — inputs `["wool"]`, output `"cloth"`, trade "the loom house" (third RNG-free duplicate-slot overwrite: shepherd, beekeeper, weaver)
- Normal shop seeding gives the loom house; the tailor's `cloth` input resolves to nearest seller — village cloth beats the import
- The F2 wool chain closes: fold → loom → tailor

## Quality gates
- `cargo test --lib -p game2-core`: 407/407 (2 new weaver_tests)

## Deferred
- Trader price-rise on local competition (needs per-seller price index)
