# PR #159 — Power recognizes power

https://github.com/garysieling/game2/pull/159

## What changed
- `OrganizedPower` ECS component on `self.org` ("The Harvesters")
- `record_power(quality)` — house meetings ≥3 → Transformational, <3 → Overreach; player-brokered matches → Transactional
- `power_tick` (weekly): `step_idle` atrophy 0.02/wk + `lord_esteem` EMA toward capacity (8%/wk — the estimate lags the truth)
- `Goal::CollectTax` on the player: esteem >0.3 → halved ("the lord learns to tread lightly"), >0.6 → collector passes by ("he knows an organized man when he sees one")

## Quality gates
- `cargo test --lib -p game2-core`: 413/413 (2 new power_tests)

## Deferred
- Sanction/feast power bumps; per-org `power_estimate` matrix (single org for now)
