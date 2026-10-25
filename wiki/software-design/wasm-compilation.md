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
