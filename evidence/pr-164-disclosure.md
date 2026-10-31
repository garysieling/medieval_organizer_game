# PR #164 — A truth sits unsaid until it ripens

https://github.com/garysieling/game2/pull/164

## What changed
- `Game.pending_discovery`, `disclosure_made`, `cured` (incremented on each successful herb-wife treat)
- `discovery_tick` (every 30d): seeds from live state — `translator` → "the lord's fifth is really a quarter"; `cured ≥ 3` → "the old cures work, doctrine or no" — ripens 14 days → disclosure
- Response rides `lord_esteem`: >0.4 → "the village stands with him" (ally-buffer); ≤0.4 → 3× `Blamed` → F13's ladder (stocks/exile path — the emergent Galileo ending)

## Quality gates
- `cargo test --lib -p game2-core`: 422/422 (2 new disclosure_tests)

## Deferred
- Whisper/pamphlet/pulpit disclosure-choice; guild-patron buffers; sky-omen discovery kind
