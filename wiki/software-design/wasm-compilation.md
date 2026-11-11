# WASM Game Compilation

**Category:** Architecture / Deployment
**Source:** Backlog item (Medium) — "WASM game compilation: browser-based deployment."
**Date:** 2026-10-25

## Description

Compile the game to WebAssembly for browser play — the distribution path that gets the game in front of people with zero install. The architecture already supports it: the sim is pure Rust, render is a thin client over it.

## Why WASM fits this architecture

- **Sim is pure Rust** — `game2-core` has no native-only dependencies in its core (no OS-specific IO in the tick loop); it can target `wasm32-unknown-unknown`
- **Render is a client** — the isometric renderer draws to a pixel buffer; in WASM that's just a `<canvas>` ImageData/blit — no GPU API required
- **Deterministic** — WASM is single-threaded and deterministic-friendly; the sim model carries over
- **Headless too** — agents/testing already run the sim without render; in a browser the render layer just attaches

## What's needed

| Component | Effort |
|---|---|
| wasm-bindgen / wasm-pack setup | standard toolchain |
| Canvas renderer adapter | small — the software rasterizer outputs pixels; blit to canvas |
| Input adapter | DOM events → sim input stream |
| Audio | Web Audio from the audio-cue events (`sfx-ambient.md`) |
| Persistence | IndexedDB for save/load (`datomic-save-load.md`) |
| Asset loading | bundle assets or stream via fetch |

## Tradeoffs

- **Performance:** WASM is near-native for compute-bound work (the sim); the heavy part is fine. Memory is the real constraint — large worlds need care with linear memory
- **Determinism across hosts:** JS's Number is IEEE-754 like Rust f64 — mostly safe, but float ops through JS shims could drift; keep math in Rust
- **No threads (easily):** SharedArrayBuffer needs COOP/COEP headers; single-threaded sim is simplest — acceptable since the sim isn't parallel anyway
- **Debugging:** harder than native — keep a native target for dev; WASM is a deployment target, not the dev environment

## Recommended approach

- Ship as **a separate thin WASM crate** (`game2-wasm`) wrapping `game2-core` — the core stays platform-agnostic
- Deliverable: a static web page — game + canvas + minimal UI shell; no backend needed (self-contained)
- Keep native as the primary dev/test target; WASM is the distribution demo

## Rating

Worth doing once the game is playable — a zero-install demo is a huge reach lever, and the architecture makes it cheap.

## Status (as of #818's first stage)

The renderer port landed: `game2-web` now links `game2-client` for wasm32
(`minifb` is the only desktop-only dep, target-gated in the client's
Cargo.toml). `WasmGame` grew the page's pipeline:

- `asset_manifest()` - the paths `Art::load` will ask for, found by
  dry-running the load with a fetch that just records names
- `provide_asset(rel, bytes)` - the page fetches each over HTTP and
  hands the bytes back; a miss draws the desktop's placeholder path
- `open_view()` - builds the same `View` the window runs
- `render()` + `frame_ptr()/frame_len()/frame_width()/frame_height()` -
  the full `View::render` output as RGBA pixels, zero-copy over
  `wasm.memory.buffer`; the camera runs the window's own sequence
  (focus jumps, comfort-box follow, event cam)
- `index.html` + `main.js` - fetch, blit via `ImageData`, drag-pan and
  wheel-zoom wired; serve the repo root and open `/game2-web/`
- `.github/workflows/wasm.yml` - wasm32 build + `game2-web` tests in CI

Still to come, per the grill's staging: the ws client against
`game_server` (the shipped mode), Web Audio from `sfx` cues, IndexedDB
save/load, and the picker/prologue card.
