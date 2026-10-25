# Trolley Problem Mechanics — Moral Dilemmas in Game Design

**Category:** Design / Narrative Mechanics
**Source:** Backlog item (High) — "Trolley problem game mechanics: moral dilemmas, ethical choices in gameplay."
**Date:** 2026-10-25

## Description

How moral dilemmas work as *systemic events* in the sim — choices with real tradeoffs and no clean answer, where the "right" choice depends on which villagers you care about. The trolley problem structure is: a constrained choice where every option costs something real, and the cost lands on named people.

## Why systemic beats authored

- An authored dilemma is a content item — the player reads it, picks, moves on
- A *systemic* dilemma lands differently because the player **knows the people** — diverting the flood to the north field saves the mill but drowns the baker's orchard, and the baker is the one whose family storyline you've followed
- Emergent dilemmas can't be "solved" by reading a wiki — the answer depends on local state

## Dilemma generators already in the sim

| Situation | The tradeoff |
|---|---|
| Famine | feed the workers or the children; eat the seed grain |
| Epidemic | quarantine a neighborhood (saves the town, traps the sick) |
| Siege | sally out (risk men) vs ration (risk all) |
| Enclosure | efficiency + lord's favor vs commons families displaced |
| Heresy | tolerate (orthodox neighbors' ire) vs suppress (community bonds vs. state) |
| Conscription | send the smith's apprentice or the miller's son |
| War levy | stay loyal to a cruel lord or revolt into uncertain consequences |

## Design principles

- **Named people, real stakes** — a dilemma only bites when the cost is a villager you've watched (drives emergent-storytelling resonance)
- **No moral score** — don't reward "good" choices; consequences play out through the sim (a kind choice that starves the village reads correctly, not as a hidden morality deduction)
- **Delay and uncertainty** — the best dilemmas let you *defer* at a mounting cost; deferral is a strategy with its own price
- **The allegory does work** — many dilemmas are the extraction regime's costs made personal (`medieval-allegory-power-dynamics.md`)

## Implementation

- Dilemma = an event with **constrained options** each carrying predictable-but-costly consequences + unknowns (surprise, reputation, downstream cascades)
- Draw from threshold events (see `emergent-storytelling.md`, `small-events-big-impact.md`) — a dilemma is a threshold event that pauses for a decision
- UI: dilemma dialog surfaces the *people affected* (villager portraits + relationships), not just stats — the dilemma is felt, not computed

## Related

- `emergent-storytelling.md`, `small-events-big-impact.md`, `comeback-events.md`, `medieval-allegory-power-dynamics.md`

## Rating

The narrative gold — dilemmas are where the sim and the story fuse.
