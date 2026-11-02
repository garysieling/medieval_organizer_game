# PR #175 — A voice with a flourish on it

https://github.com/garysieling/game2/pull/175

## What changed
- `REGISTERS` (8 flourished small-talk lines) + `INSULTS` (5 gossip-rung lines) in `organizing.rs`
- `passing_remark(n)` samples a register every 4th pick — deterministic, no RNG consumed
- `Gossip` sanction rung says "They call me {insult}." — pure voice, zero mechanics

## Quality gates
- `cargo test --lib -p game2-core`: 443/443 (1 new voice_test)
