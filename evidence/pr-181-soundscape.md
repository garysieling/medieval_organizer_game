# PR #181 — The cues rise with the days

https://github.com/garysieling/game2/pull/181

## What changed
- `SfxCue { kind, x, y, intensity }` + `Game.sfx` queue + `emit_sfx` + `sfx_tick`
- Ambient bed: "rain" while `precip` falls (every 20d), "anvil" the smithy's rhythm (every 40d)
- Moments: "festival" on feast days (tick%90==45), "wonder" when a sky event burns
- `UiModel.sfx` per frame; queue clears each `update` — per `sfx-ambient.md` ("sim raises cues, the audio layer maps them")

## Quality gates
- `cargo test --lib -p game2-core`: 452/452 (2 new sfx_tests)

## Deferred
- Positional pan/distance-mix (the audio layer reads x,y,intensity); actual audio output
