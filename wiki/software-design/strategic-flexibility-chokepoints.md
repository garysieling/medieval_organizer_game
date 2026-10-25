# Strategic Flexibility & Chokepoints

**Category:** Design / Strategy
**Source:** Backlog item (High) — "Strategic flexibility & chokepoints: defensive play, multiple approaches."
**Date:** 2026-10-25

## Description

How the world supports different play styles — turtle up, expand aggressively, play trade, play politics — and how geography creates the *real* strategic texture: the chokepoints, buffers, and corridors that make location matter.

## Multiple viable strategies

| Style | How it works in our systems |
|---|---|
| **Turtle** | defensible geography (hill, island, narrow valley) + self-sufficiency + walls; wins by outlasting |
| **Expand** | fertile river valleys + surplus → more settlements → scale; wins by production mass |
| **Trade** | chokepoint/crossroads position → tolls and commerce → leverage without land |
| **Political** | diplomacy, marriage, subversion — power without territory (delegation/influence mechanics) |
| **Rush/raid** | mobility + surprise — take resources before they can respond (raids are real in medieval context) |

## Chokepoints as the real map

- **Natural chokepoints** come from world gen: a mountain pass, a river ford, a harbor mouth, a bridge — whoever holds it controls flow (`world-generation-exploration.md` generates these)
- **Built chokepoints** — a castle at a pass, walls across a valley, a fortified bridge: fortification *creates* chokepoints where geography permits
- **The economy of the chokepoint** — a ford or pass is also a toll-booth opportunity; chokepoints are strategic *and* economic assets
- **Chokepoints are relative to movement** — the map's real topology is the *path graph*: a "chokepoint" is a cut vertex in the paths between resources/markets

## Defensive play (turtling done right)

- Defense should be *viable but not free*: a turtle wins by absorbing pressure, but pays in opportunity cost (land, trade, access)
- Attrition is the turtle's friend — siege is a starvation race (`non-rps-combat.md`); a supplied, walled position forces the attacker to bleed time and logistics
- Turtles still need *something* — a purely defensive position with no production collapses; defense needs an internal economy

## Flexibility checks (anti-lock-in)

- The map shouldn't force a style: chokepoint-heavy maps shouldn't make trade impossible; open maps shouldn't make defense impossible
- Position determines *which* strategies are natural, not which are possible — a mountain region can trade through passes, it's just different

## Rating

What makes the map a *strategy* game rather than a scenery generator.
