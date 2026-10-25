# Visual Verification Protocol

**Rule:** any UI change must be verified by rendering a frame of the real code path.

```
cargo run --bin game_ui -- --dump-frame 120
# inspect evidence/frame-check-120.bmp
```

Checks to run on each frame:
- Single map (no duplicate projections)
- Buildings anchored bottom-center on tiles, not floating
- Villagers not clustered/overlapping
- Text fits panels (no truncation)
- No strobing effects
- Buttons only enabled when actionable (proximity)
- Sprites aligned to isometric grid

Failing any check = visual bug. Fix before commit.
