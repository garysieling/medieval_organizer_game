# Realistic Relationship Modeling (Beyond RimWorld)

**Category:** System Design / Simulation
**Source:** Backlog item — "RimWorld has good dialog between characters but relationship system needs improvement. Incorporate attachment theory, social dynamics, history, context." Implemented stub: `game2-core/src/game/relationships.rs` + `sim/psychology.rs` (attachment emergence); see `wiki/literature-reviews/sprint-1-psychology-economics.md`.

## Description

Relationships as a persistent, multi-dimensional, history-bearing state — not a single opinion score. RimWorld's model (opinion + social log of interactions) produces good barks but shallow arcs: relationships fluctuate around an interaction mean and forget context. This design adds *structure*: attachment style (emergent per the psych lit review), relationship history, context, and multi-dimensionality.

## Dimensional model

Each directed edge A→B carries:

| Dimension | What it is | Volatility |
|---|---|---|
| **Attachment** | emergent style toward B (secure/anxious/avoidant) — from `sim/psychology.rs` anxiety-vs-warmth learning, NOT assigned | slow, forms over shared history |
| **Trust** | belief B keeps commitments — moves on kept/broken contracts (feeds `game/contracts.rs`) | medium |
| **Warmth/affection** | likes B's company | medium, event-driven |
| **Respect/status** | B's competence/standing relative to A's values | slow |
| **History** | timestamped salient events (fights, rescues, 1-1s, betrayals) | append-only, decays in salience not existence |
| **Context** | relationship *kind*: kin, coworker, congregant, rival — constrains what the other dims mean | structural |

RimWorld's single opinion ≈ a blend of warmth+respect only; this splits the dimensions so "respects but dislikes" (the colleague you can't stand) is representable.

## Key mechanics

- **History gates behavior:** two characters at +40 warmth behave differently if the history is "3 years of loyalty" vs "yesterday's fling" — the *record* shapes the interaction, not just the score.
- **Context constrains expression:** anxious attachment in a kin context reads as clinginess; in a work context, as over-reporting. Same attachment, different visible behavior.
- **Third-party effects:** relationships triangulate — A's bond with B shifts when B wrongs C (loyalty/jealousy/solidarity). This is where organizing (`sim/community.rs`) gets traction: 1-1s literally build relationship edges that change group structure.
- **Asymmetry:** A→B ≠ B→A. Unrequited respect, one-sided grudges — the good drama is in the asymmetry.
- **Decay with salience, not deletion:** old wounds fade in weight but resurface (a triggering event re-raises an old history entry).

## Dialog / barks

Keep RimWorld's trick — generate barks from the *most salient current state* — but feed it richer state: an anxious-attached character who's been recently slighted barks differently than a securely-attached one at the same warmth score. Bark selection = f(attachment, last salient event, context).

## Implementation notes

- `game/relationships.rs` holds the edge store (entity-pair keyed); `sim/psychology.rs` writes attachment updates; events append to history.
- `ui/graph.rs` already renders social bonds — extend edge coloring to show the dominant dimension.
- Evidence hook for testing: after N turns, dump the relationship graph; check for emergence of *clusters* (families, cliques, outcasts) — that's the sim working.

## Rating

This is the substrate for half the emergent stories the game wants — worth the multi-dimensional cost.
