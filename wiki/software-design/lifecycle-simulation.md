# Full Lifecycle Character Simulation

**Category:** System Design / Simulation
**Source:** Backlog item — birth to death character arcs. Implemented stub: `game2-core/src/game/lifecycle.rs`.

## Description

Characters as *life courses*, not static pawns: birth → childhood → apprenticeship → working life → decline → death, with each stage changing what the character *is* to the sim — their needs, capabilities, relationships, and role in the economy/community. This is the substrate for generational play (village→kingdom timescale) and for every other system that varies by age (illness, attachment, skills, inheritance).

## Life stages (the state machine)

| Stage | What changes |
|---|---|
| **Infancy/childhood** | dependent; attachment formation happens here (`sim/psychology.rs` — anxiety/warmth episodes in this window set attachment); trait seeds |
| **Apprenticeship** | skill growth fastest; master assignment sets both skills and a key relationship edge; first social standing |
| **Working life** | peak production; courtship/pairing, children, household formation; wealth/standing accrual |
| **Elder/decline** | physical skills decay (`skill-improvement-degradation.md`), knowledge/status peak — the elder is a *different kind of asset*: knowledge carrier |
| **Death** | inheritance, grief events, role vacancy — an *event generator*, not an endpoint |

## Mechanics that emerge from stages

- **Skill/knowledge succession:** a master dying without an apprentice is a real loss event — knowledge inventory *dies* with characters (herbology coupling). Villages can lose crafts; this is a real medieval dynamic and an emergent drama.
- **Age-structured population:** the sim's birth/death rates produce a real age pyramid — famine or plague visibly reshapes it (you see the missing generation).
- **Generational economics:** inheritance (property, tools, debts) transfers through `game/contracts.rs`/legal framework; households form/split over generations.
- **Intergenerational tension:** elders vs young on norms — a slow conflict axis that religion/community systems can exploit (reform movements are usually young).
- **Pacing anchor:** the lifecycle is the game's clock made human — events like "child comes of age" give the long sim real tempo.

## Design points

- **Stages shift behavior, not just stats:** children play (not work), elders advise, apprentices fetch — the same sim tick produces different behavior by stage.
- **Death is structured:** expected (age) vs unexpected (illness/accident/conflict) deaths produce different grief/community effects — a death's *cause* writes the story.
- **Don't simulate gametes:** lineage can emerge (who's whose child) without fertility modeling depth; keep family formation legible.

## Implementation notes

- `game/lifecycle.rs` holds the stage state machine; age ticks at the daily rate, stages gate the behavior/needs/components each system sees.
- Birth/death feeds `sim/community.rs` (household counts), economy (labor pool), and religion (rites of passage — birth/marriage/death ceremonies are cohesion events).

## Rating

Foundational — the longest-arc system in the game and the one everything else inherits from.
