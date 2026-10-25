# Evolving Music System (Techno-Style Layering)

**Category:** Audio / System Design
**Source:** Backlog item (Medium) — "music that evolves as world changes (mood, events). Techno-style: slowly adding/dropping layers over time. INITIAL VERSION: simple mood recommendation for Spotify or silent." Implemented stub: `game2-core/src/game/music.rs`.

## Description

Adaptive score as *layered stems*: the track is a set of musical layers (drone, pulse, rhythm, melody, tension) that fade in/out over minutes — techno-style evolution rather than track-switching. World state maps to which layers are active; the score drifts continuously instead of cut-scenes between "explore theme" and "danger theme."

## The layered model

| Layer | Driven by | Example |
|---|---|---|
| **Bed/drone** | base ambience — always on, sets key + mode | calm diatonic pad |
| **Pulse/rhythm** | activity level — population, work, movement | light percussion as village wakes |
| **Melody** | prosperity/mood — thriving adds warmth, struggle thins it | sparse motif when poor, fuller when rich |
| **Tension** | threat/stress — events, illness, conflict | dissonant layer bleeding in |
| **Event accents** | one-shot stingers on notable events | birth, death, festival, fire |

- Layers cross-fade over tens of seconds — the player *feels* the world shift before they can name why. That's the design goal: music as ambient telemetry.
- Seasonal/weather modulation on the bed (minor shift in winter) couples to `sim/climate.rs`.

## Phasing

- **Phase 0 (backlog's initial version):** mood classifier → "recommend Spotify vibe or stay silent." No asset pipeline needed — a `mood(world)` function emitting {calm, busy, tense, grieving, festive} for a human/playlist to interpret. `game/music.rs` can implement the classifier now.
- **Phase 1:** discrete layer assets (looped stems per layer per mood-family), mix engine = gain per layer.
- **Phase 2:** generative layer — procedural variation so the score never loops identically (the game's emergent aesthetic argues for it).

## Design points

- **The map IS the mix:** different regions/camera focus bias the layers — your village's music vs a distant war camp's music, one mix engine.
- **Music as narrator:** a death in an NPC family adds a grief layer only if the player knew them (relationship coupling) — music personalized by sim state.
- **Restraint:** the system must be able to say *nothing* — silence is a layer state too.

## Implementation notes

- Phase 0 = one function reading world mood + telemetry; do that first, it's cheap and unblocks the vibe.
- Layer count small (4-5); the trick is the cross-fade, not the layer count.

## Rating

Medium priority is right — the classifier is worth building now, the full stem mix is a polish item.
