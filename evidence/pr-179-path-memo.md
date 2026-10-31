# PR #179 — A path's the same all day

https://github.com/garysieling/game2/pull/179

## What changed
- `path_cache: HashMap<(from,to), Option<Vec>>` + `walk_memo` — the per-send BFS was the hot line; paths remembered until the day turns (obstacles only change on day boundaries)
- Swapped into `send` + `send_tile_walk`, the two hot call sites

## Quality gates
- `cargo test --lib -p game2-core`: 450/450 (1 new perf_test)

## Deferred
- Sprite cache; flock batch-separation (the path memo was the measurable hot line)
