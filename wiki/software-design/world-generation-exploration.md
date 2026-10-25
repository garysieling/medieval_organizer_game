# World Generation & Exploration

**Category:** Design / World Systems
**Source:** Backlog item (High) — "World generation & exploration: procedural map, discovery mechanics."
**Date:** 2026-10-25

## Description

How the world is generated and what exploring it feels like — the map must be interesting to live in (defensible, farmable, trade-shaped) and to reveal gradually (villagers don't know what they haven't seen — ties to `fog-of-war-memory-fade.md`).

## Generation layers (in dependency order)

1. **Geology/elevation** — heightfield → ridges, valleys, plateaus; drives everything downstream (drainage, settlement sites, chokepoints)
2. **Hydrology** — rivers flow downhill, lakes pool, floodplains mark fertile land; rivers are both resources AND borders
3. **Climate gradient** — latitude/rainfall bands bias soils and crop types (feeds `climate-change-simulation.md` baseline)
4. **Soil/fertility** — derived from geology+water+climate: floodplain best, hilltops worst — farming sites pick themselves
5. **Resources** — forests (timber/fuel/forage), ore outcrops, stone, clay, herbs (`herbology`), trade-route chokepoints
6. **Regions** — Voronoi-ish territories partitioned by watersheds/ridges (natural borders, not grids) → feeds `regional-ruler-personality.md`
7. **Settlements** — placed where fertility + water + defensibility overlap; villages seed, towns grow at crossroads/ports
8. **Roads** — paths beaten between settlements (desire-path generated, not drawn) — trade routes emerge from connectivity

## Discovery mechanics

- **Knowledge is per-villager** — a map edge is a villager's memory (`fog-of-war-memory-fade.md`); the player's "map" is what *any* villager knows
- **Scouting & trade reveal** — send a cartographer, follow a trader, host a pilgrim: routes to knowledge are social/commercial, not a fog-reveal button
- **The unknown has contents** — unexplored regions have real state (systems run there); discovering a region means meeting functioning places, not empty terrain

## Design rules

- **Readable geography** — hills look like hills, rivers like rivers: the isometric layer (`isometric-pixel-art-layers.md`) must make geography legible at a glance
- **Geography as fate** — a village's destiny is mostly where it sits; placement decides wealth before policy does (the geography-is-destiny thesis, shared with medieval economics lit)
- **Defensibility is generated, not assigned** — chokepoints come from terrain (passes, fords, harbors) — see `strategic-flexibility-chokepoints.md`

## Related

- `climate-change-simulation.md`, `regional-ruler-personality.md`, `fog-of-war-memory-fade.md`, `dynamic-map-evolution.md`, `village-to-kingdom-scale.md`

## Rating

The substrate — good generation makes every downstream system richer.
