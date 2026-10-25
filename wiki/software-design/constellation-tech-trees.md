# Constellation Tech Trees

**Category:** Design / Progression
**Source:** Backlog item (Medium) — "Constellation tech trees: non-linear research visualization."
**Date:** 2026-10-25

## Description

The *visual/structural* form of the tech system (see `tech-tree-design.md` for the model): not a linear tree or a grid — a constellation of related techniques laid out spatially, where proximity means relation and the "sky" fills in as knowledge is discovered.

## Why constellation

- A linear tree forces ordering that isn't real (you don't need pottery to get brewing)
- A constellation preserves *clusters* — related techniques group naturally (weaving cluster, metallurgy cluster, healing cluster) — and shows the *topology* of knowledge
- It's *discoverable* — the sky fills in as you learn; unexplored regions are literally dark sky

## Layout

- **Nodes = techniques**, positioned by craft domain (agriculture, metallurgy, textiles, healing, building, writing, trade, magic…)
- **Proximity = relation** — techniques that share inputs/infrastructure cluster together (all weaving tech near each other; herbalism near healing)
- **Edges = enabling links** — technique A *enables* technique B (not strictly "requires" — softer dependency)
- **Brightness = mastery** — discovered techniques glow; skilled/mastery level brightens; unexplored regions are dark
- **Motion** — a technique "rising" when unlocked; the constellation fills in over play — the map of knowledge is also a progress display

## Constellations per domain

- **Crafts cluster** — textiles (`textiles-tech-tree.md`), metallurgy, pottery, woodwork, building
- **Food/agri cluster** — farming rotations, brewing, baking, preservation
- **Healing cluster** — herbology (`herbology-apothecary-tree.md`), medical tech (`technology-medical-options.md`), surgery
- **Letters cluster** — scribes, printmaking (`printmaking-tech-tree.md`), record-keeping → bureaucracy
- **Trade cluster** — markets, credit, contracts (`legal-framework-contracts.md`)
- **Music/cluster** — instruments, composition (`evolving-music-system.md`)
- **Magic cluster** — traditions and schools (`novel-magical-powers.md`)

## The discovery feel

- A technique isn't *researched*, it's *discovered* — practitioners hit it by working, a book reveals it, an apprentice brings it
- The "sky" is regional — different regions light different parts (`tech-tree-design.md`'s uneven diffusion)
- The constellation view is the player-facing visualization of the underlying tech state — `ui/tech_view.rs`

## Rating

A distinctive UI/visualization for the tech model — evocative and it *means* something.
