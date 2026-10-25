# Playthrough Video Generation

**Category:** Tooling / Marketing
**Source:** Backlog item (High) — "Playthrough video generation: automated gameplay recording."
**Date:** 2026-10-25

## Description

Automatically record playthroughs to video — for documentation, marketing, debugging, and the "watch the village live" experience. The sim produces the footage; the pipeline makes it a video.

## The generation pipeline

```
run sim → capture frames (render each tick or every N ticks) →
assemble → optionally narrate/score → output mp4/gif
```

- **Determinism makes it reliable** — a recorded run replays exactly; the video is a deterministic artifact, not a screen recording (`deterministic-simulation-ai-safety.md`)
- **Headless render** — `ui/isometric.rs` renders to a pixel buffer; no GPU needed; already producing BMP evidence frames
- **Story-annotated video** — `emergent-storytelling.md` events can be overlaid as captions/titles (the video is also a *chronicle*)

## Use cases

| Use | What it captures |
|---|---|
| **Evidence/dev** | reproducing a bug, showing a system's behavior, documenting a sprint |
| **Marketing** | "watch a village live a year" timelapse — the game's best demo |
| **Chronicle** | a run's emergent story rendered as a mini-documentary |
| **Testing** | playtesters' sessions recorded for review (feeds the Tester agents) |

## Design considerations

- **Every-N-frames capture** — capture at tick intervals, not real-time; timelapse is the natural format for a sim
- **Camera scripting** — pan/zoom to follow the *story* (the interesting village, the battle, the festival); a camera that just sits on spawn is boring
- **Caption overlay** — pull event titles into the video as subtitles; the video narrates itself
- **Length discipline** — a timelapse is minutes; compress a year into watchable form

## Implementation notes

- `game2-core` render → frame buffer → encode via ffmpeg (or a pure-Rust encoder); binaries `sim500`/`mega_sim` are the capture harnesses
- The agent fleet can generate these as artifacts — a successful run can *emit its own video*

## Rating

High-leverage marketing + dev tool — a run's output that sells itself.
