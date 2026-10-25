# SFX & Ambient Audio Design

**Type:** Audio Reference
**Source:** Backlog item (Low, artistic) — "SFX/ambient sounds: audio design."
**Date:** 2026-10-25

## Description

The audio design spec for a medieval village sim — ambient soundscape + reactive SFX. Priority: Low (the sim is the game, audio is polish — but the *ambient* layer matters a lot to the cozy aesthetic).

## The ambient bed (the important part)

- **Village as a soundscape:** chickens, dogs, hammer-on-anvil rhythm, church bell hours, wind through thatch, water, market murmur — the "living world" feel comes mostly from this layer
- **Time-of-day cycle:** dawn birds, midday labor, evening hearth-tavern hum, night crickets — the sound of a day
- **Seasonal variation:** winter wind vs summer insects; harvest time clatter
- **Distance mixing** — sounds fade with camera distance; zoomed-out = murmur, zoomed-in = specific loops

## Reactive SFX (tied to events)

- Economic: coin clink, market haggling, hammering
- Villager: footsteps, chatter, laugh, cry, work-grunts
- Events: bell alarms, festival music, mob rumble, fire crackle, rain
- Combat: steel, shield-thud, horses, cries — rare, loud, alarming
- UI: parchment, ink-scratch, button ticks (period-appropriate UI sounds)

## Music

- See `evolving-music-system.md` — music evolves with the world's state; ambient should hand off to music at moments (festivals, battles, ceremonies)

## Implementation

- `engine/audio.rs` (audio layer) — decoupled from sim; sim raises *cues* (event type, position, intensity), the audio layer maps cues → sounds
- Sources: CC0 libraries (Freesound, BBC Sound Effects archive — public domain status varies, verify), field recordings, generated audio
- Spatial: position-based pan/level for isometric view

## Rating

Low priority but high atmosperic payoff — a quiet game with a great ambient bed feels expensive.
