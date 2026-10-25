# Village to Kingdom Scale

**Category:** Design / Scope
**Source:** Backlog item (High) — "Village to kingdom scale: how the game scales up."
**Date:** 2026-10-25

## Description

How the game scales across levels — the player starts at village scale and the world contains kingdoms. The core question: what's the player's role as the world widens? The answer: the *level of agency changes*, not the interface.

## The scale ladder

| Tier | Scope | Player role | What's simulated |
|---|---|---|---|
| **Homestead** | a household | direct patronage of a family | individuals in detail |
| **Village** | ~50-200 people | patron/influence — shape conditions | villagers + their relationships in full |
| **Town** | ~500-2000 | steward appointment, policy | villagers aggregate somewhat; districts |
| **Domain** | several settlements | lordly policy, delegation tree | settlements + representatives |
| **Kingdom** | realm | royal policy, war/succession | regions, factions, diplomacy |

## The agency transformation

- At village scale, the player uses `single-character-indirect-control.md` + `indirect-control-mechanics.md` — intimate, person-level
- At town+, direct villager influence fades; the player operates through `delegated-agency-characters.md` — appointing stewards, setting laws, funding projects
- At domain/kingdom, the game becomes a `player-agency-victoria-model.md`-style play: policy, diplomacy, succession — the villagers resolve into factions and aggregates

## Design rules

- **You can't scale a person** — above a few hundred villagers, individual tracking gives way to aggregates; but the *systems* don't change — the aggregate is computed from real villager state, not faked
- **Each tier is still the same world** — a kingdom isn't a different game mode; it's the same sim where the player operates at a coarser altitude
- **Loss of intimacy is the cost** — a player who loved knowing every villager loses that at scale; the design should make delegation feel like *governing*, not abandoning
- **The world is already full-scale** — the map always contains kingdoms; the player just *starts* at village level and can grow — "scale up" is the player's progression, not a feature to unlock

## Growth mechanics

- A village grows by: births, migration (drawn by prosperity/safety/land), marriage, absorption of smaller settlements
- Transition triggers: population thresholds, economic complexity, political incorporation (village becomes a town when it's *big enough to matter*)
- Failure reverses: a town can collapse back to a village (plague, famine, conquest) — scale is not monotonic

## Related

- `indirect-control-mechanics.md`, `delegated-agency-characters.md`, `player-agency-victoria-model.md`, `world-generation-exploration.md`, `regional-ruler-personality.md`

## Rating

The scope question — the thing that turns a village sim into a *world* game.
