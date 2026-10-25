# Dynamic Map Evolution

**Category:** Design / World Systems
**Source:** Backlog item (Medium) — "Dynamic map evolution: map changes over time."
**Date:** 2026-10-25

## Description

The world map isn't static backdrop — it changes over play: rivers shift, forests spread or are cleared, soil exhausts, coasts erode, settlements grow/decline, borders move. The map should feel *lived in* by time itself.

## Evolution layers

| Layer | Timescale | Drivers |
|---|---|---|
| **Ecological** | decades-centuries | forest regrowth on abandoned land, soil exhaustion from over-farming, succession after fire, marsh drainage |
| **Climatic** | years-decades | drought bands shift, growing season changes (`climate-change-simulation.md`) |
| **Geomorphic** | decades+ | river meanders/avulsions (a ford moves, a village loses its crossing), coastal change, landslides |
| **Anthropogenic** | years-decades | deforestation for fuel/fields, field pattern changes, new roads as traffic uses them, wall building, terracing |
| **Settlement** | years | villages grow into towns, shrink to ruins, relocate after disaster |
| **Political** | events | borders shift with conquest/inheritance |

## Design principles

- **Change is visible and legible** — the player sees a forest creep back onto abandoned land, a river carve a new course; the map *tells* time's story
- **Mostly gradual, occasionally dramatic** — slow drift punctuated by events (a flood reroutes the river; a fire resets the forest) — same cadence as `small-events-big-impact.md`
- **Anthropogenic is the interesting axis** — the player's choices reshape the map: deforestation → erosion → lower fertility → decline; enclosure → field pattern changes; a village's footprint literally alters its surroundings
- **Ruins persist** — an abandoned settlement leaves a ruin (archeology in the map): history is written *on* the land

## Mechanic mapping

- `game/terrain.rs` holds the physical map; evolution = slow state transitions on tiles (soil, vegetation, water) driven by use + climate + time
- Settlement state (pop, wealth) drives footprint growth/shrink; ruins as persistent tile types
- Borders from `factions.rs` control — pure data, map-side rendering
- Discovery layer (`fog-of-war-memory-fade.md`) means villagers' *memory* of the map can be outdated — the world changes faster than knowledge of it

## Rating

Medium — a multiplier on everything else (makes the world feel real) but not a load-bearing new system.
