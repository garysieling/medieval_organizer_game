# PR #173 — The dark keeps them

https://github.com/garysieling/game2/pull/173

## What changed
- `cave_pos` (36,33) + `TownRole::Hermit` on villager 22 — `Problem::TheDark` ("the dark keeps me", permanent, all desires wiring)
- Deep `MushPatch` at (35,33) — the best patch, pushed last so wood-poison indices hold
- `Goal::TalkTo` on the hermit → "The pale one grows where the light dies — it's the truest of them all." + Joy
- Render: dark hollow, desaturated rim
- Latent ordering fix: `add_grievance` ran before `InnerLife` existed — moved to after the InnerLife pass

## Quality gates
- `cargo test --lib -p game2-core`: 440/440 (1 new cave_test)
