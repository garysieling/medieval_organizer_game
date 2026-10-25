# Tech Tree & Progression Design

**Category:** Design / Progression Systems
**Source:** Backlog item (High) — "Tech tree & progression design: research, upgrades, unlock mechanics."
**Date:** 2026-10-25

## Description

How technologies/unlocks are structured: not a Civ-style linear tree but a *constellation* — clusters of related techniques that spread through craft, guild, and practice rather than a research menu. See `constellation-tech-trees.md` for the spatial formulation.

## Model

- **Nodes = techniques** (not "technologies"): a way of doing — four-field rotation, charcoal burning, glazed pottery, water-powered fulling. Each has prerequisites (materials, tools, infrastructure, knowledge).
- **Unlock conditions are material, not abstract:** a technique becomes available when its inputs exist in the world — a bloomery unlocks smelting techniques, not "reach Iron Age." Tech is *grounded in what the village can do*.
- **Knowledge diffuses, it isn't purchased:** techniques spread via guilds, apprenticeships, migration, captured artisans, books (printmaking → literacy → faster diffusion).
- **Progression is uneven:** regions develop asymmetrical techs (regional staples bias → regional specialties). No tech "race" — the map looks like a patchwork of competencies.

## Tree shape (the constellation)

- **Clustered around crafts:** textiles (see `textiles-tech-tree.md`), metallurgy, agriculture, building, medicine (`technology-medical-options.md`), herbology/apothecary (`herbology-apothecary-tree.md`), printing (`printmaking-tech-tree.md`), tea (`tea-tech-tree.md`), music (`evolving-music-system.md`).
- **Cross-links where history had them:** printing → literacy → record-keeping → bureaucracy → law (`legal-framework-contracts.md`); metallurgy → tools → agriculture yields.
- **Soft gates:** a technique isn't binary-locked — villagers attempt it with poor efficiency at low skill; mastery emerges from practice (`skill-improvement-degradation.md`).

## Anti-patterns to avoid

- **Research points as currency** — tech shouldn't be bought from an abstraction pile; it's discovered in context
- **Linear eras** — no "reach Age II" gates; progress is local and uneven
- **Tree as checklist** — depth per technique (mastery, quality tiers) beats breadth of options

## Progression layers (scale)

Tech drives `village-to-kingdom-scale.md` progression: a village's techniques determine what it can produce → trade position → wealth → political weight. Tech *is* the scale ladder's mechanism.

## Implementation

- `game/tech_tree.rs` + technique nodes as data (provides: outputs; requires: inputs/buildings/knowledge); consistent with the mod contract (`modular-architecture-everything-is-a-mod.md`).
- Knowledge as villager attribute + guild attribute; diffusion over the relationship/trade graph.

## Rating

Move past the Civ paradigm — technique constellations fit a society sim.
