# Realistic Natural Structures

**Type:** Visual Reference / World Design
**Source:** Backlog item (High) — "natural environments: real geography, geology."
**Date:** 2026-10-25

## Description

How natural features should look and behave — the visual/geological realism baseline. Real geography has structure: drainage networks, soil horizons, vegetation zones, erosion patterns. Getting the *physics* of landscape right makes everything else look correct.

## The physics of landscape (visual rules)

- **Water shapes everything** — rivers carve V-valleys; floodplains are flat and fertile; meanders create oxbow scars; deltas spread. River valleys have characteristic *profiles* (narrow at source, broad downstream)
- **Geology reads in the rock** — bedrock outcrops on steep slopes/ridges; soil depth varies with slope (thin on ridges, deep in valleys); different rock = different landform character
- **Vegetation follows water + soil** — forests on moist slopes, meadows in floodplains, scrub on dry ridges, marsh in poorly-drained lowlands; tree line on altitude
- **Slope controls land use** — flat → fields, moderate → pasture/orchards, steep → forest/moor, very steep → unusable or terraced

## What makes it feel *real*

- **Edges, not grids** — natural boundaries are gradual (forest→scrub→grassland→marsh transition tiles), not abrupt biome switches
- **Pattern at multiple scales** — from the map view: drainage dendrites, elevation contours, valley networks; zoomed in: soil texture, field patterns, hedgerows
- **Use marks the land** — fields plow into the soil, paths wear in, clearings are worked, old forest is denser than new growth
- **Asymmetry** — nature isn't symmetric: a north-facing slope differs from south-facing (snow retention, vegetation, use); valleys have character

## Mechanical grounding (not just visuals)

- Geology → soil → fertility → settlement placement (`world-generation-exploration.md` uses this pipeline)
- Slope + drainage → flood risk, erosion, irrigation
- Vegetation zones → forage, timber, hunting yields
- Rock type → quarryable stone, ore probability

## Art application

- Terrain transition tiles at every biome edge — no hard biome polygons
- Elevation as visible terraces/slopes (isometric layers, `isometric-pixel-art-layers.md`)
- Rivers with banks, fords, and delta/mouth detail; fields keyed to soil type

## Rating

The realism floor — if the land doesn't behave like land, nothing placed on it reads correctly.
