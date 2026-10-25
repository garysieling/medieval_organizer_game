# Isometric Pixel Art with Layers

**Type:** Art Technique Reference
**Source:** Backlog item (High) — layering for isometric pixel art.
**Date:** 2026-10-25

## Description

How to render a dense isometric pixel-art world: the classic 2:1 dimetric tile (diamond twice as wide as tall, e.g. 64×32 px) plus a strict layer stack — the technique behind the Caesar/Impressions and later isometric look the project targets.

## The tile system

- **Tile:** 2:1 diamond, typically 64×32 or 48×24 px. Grid→screen: `screenX = (x−y)*w/2`, `screenY = (x+y)*h/2 − z*height`.
- **Height:** elevation expressed as a z-offset (subtract from screenY) — this is how hills/terraces draw correctly.
- **Masking:** a sprite's pixels overlap neighbors — depth is resolved by *draw order*, not geometry.

## Layer stack (bottom → top)

1. **Terrain/ground tiles** — grass/dirt/water bases, edges handled with transition tiles
2. **Elevation overlays** — cliffs, slopes, terraces
3. **Static structures** — buildings, walls, bridges (sorted south-to-north)
4. **Decor/props** — carts, stalls, laundry lines, gardens
5. **Actors** — villagers, animals (sorted *with* structures by tile position + sub-tile offset, not a blanket top layer)
6. **Tall-mask objects** — trees/towers drawn so an actor can walk "behind" them (split-tree technique: trunk in layer 5, canopy above actors)
7. **Effects** — smoke, fire, magic, weather
8. **UI/fog** — fog of war tint, selection, indicators

## The sort rule that makes it work

Draw order isn't per-layer — it's per-*tile*: for each tile south-to-north (then within a tile, ground → structure → prop → actor). "Behind a building" works automatically; floating-head bugs are sort-bugs, not layer-bugs.

## Implementation in game2

- `ui/isometric.rs` implements the engine-level renderer; this entry is the art/technique spec it implements. BMP layer-separated renders already produced in evidence/.
- Minimap (`ui/minimap.rs`) is a coarser orthographic projection of the same tile model — no separate scene.

## Palette & readability

- Cohesive warm earth palette (see `medieval-thematic-research.md`)
- Silhouette-first buildings — a bakery reads from its roofline + chimney, not its pixels
- Lighting: fixed NW-up sun, consistent shadows bake depth into 2D art

## Rating

Foundational art spec — ties the renderer to an achievable style; isometric.rs already implements the core.
