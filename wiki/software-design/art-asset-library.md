# Art Asset Library Strategy

**Category:** Pipeline / Art
**Source:** Backlog item (High) — "Art asset library: asset acquisition, creation, or generation."
**Date:** 2026-10-25

## Description

How the project gets its art: the strategy for acquiring/creating/generating the tilesets, sprites, portraits, and UI elements the game needs — at AI-dev scale, not artist-studio scale.

## The strategy mix

| Source | Best for | Constraint |
|---|---|---|
| **Public-domain/open asset packs** (Kenney, OpenGameArt) | placeholders, prototypes, simple icons | CC0/CC-BY only; quality varies |
| **Generative image models** (local diffusion, or API) | concepting, portraits, buildings, textures | style consistency is hard; needs curation |
| **Pixel-art generation** | isometric tiles, sprites | diffusion is weak at true pixel-art; may need sprite tools or hand-cleanup |
| **Commissioned pixel artists** | hero assets — the village, key sprites, the UI | costs real money; the smallest viable scope |
| **In-engine generation** | terrain, procedural textures | some assets can be procedurally produced |

## Recommended pipeline

1. **Placeholder-first** — open assets to get the game *running*; don't block on final art
2. **Style guides** — `medieval-thematic-research.md` + `isometric-pixel-art-layers.md` define the palette/silhouette rules so generated art has a target
3. **Generate → cull → clean** — the Aesthetic Consultant agent reviews generated batches; only the best survive; pixel-level cleanup where needed
4. **Hero commission** — a small set of commissioned pixel art (the village centerpiece, the UI frame, the key portraits) gives the game a *look* while generated/placeholder art fills the volume
5. **Procedural for terrain** — landscape tiles generated, not drawn

## Asset types needed

- Terrain tiles (with transitions)
- Building sprites (per type + per region, `regional-architectural-variety.md`)
- Character sprites (villagers, classes, ages — `lifecycle-simulation.md`)
- Portraits (character sheet UI, `character-sheet-ui.md`)
- UI elements (panels, icons, frames)
- Props/decor (the lived-detail layer)

## The indexing problem

- The asset library is also a *knowledge problem*: tag by (type, region, era, palette, mood) so agents/generators can find or produce the *right* asset — `agent_kb`-style metadata on each asset
- The library is a first-class deliverable for the Art Director role; generation without a curatorial pass produces sludge

## Rating

A real production problem — the strategy is hybrid (open + generated + commissioned heroes), not one source.
