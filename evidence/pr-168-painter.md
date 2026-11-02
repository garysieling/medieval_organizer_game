# PR #168 — The paint shop's work shows

https://github.com/garysieling/game2/pull/168

## What changed
- `Vocation::Painter` — inputs `["pigment","wood"]`, output `"paint"` @8g, "the paint shop" (swap5 + guaranteed shop for Weaver/Perfumer/Painter — the 85% roll could strand a feature)
- `painter_tick` (every 14d): a pot of paint → `painted` marker (org's hall first → `record_power`); fades at +365d → repeat commissions
- Render: heraldic trim above the door
- **Structural fix:** `visible_log` pins important entries (illness/death/exile/disclosure/stocks/born) to the top — the recurring flu-eviction (7×) solved once, not per-feature

## Quality gates
- `cargo test --lib -p game2-core`: 431/431 (2 new painter_tests)
