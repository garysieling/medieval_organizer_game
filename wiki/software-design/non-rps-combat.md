# Non-RPS Combat System Design

**Category:** Design / Combat
**Source:** Backlog item (High) — "Non-RPS combat systems: alternatives to rock-paper-scissors counters."
**Date:** 2026-10-25

## Description

Alternatives to rock-paper-scissors unit counters — combat driven by *circumstance* (terrain, morale, supply, leadership, surprise) rather than unit-type multipliers. In our medieval society sim, combat should feel like war did: won by logistics, morale, and position more than troop rosters.

## Why avoid RPS

- RPS units abstract combat into a counter-matching minigame — the sim becomes about the counter table, not the situation
- Medieval combat wasn't about "counters" — it was about *cohesion, ground, morale, and who showed up fed*

## Circumstance-driven combat (the alternative)

| Factor | Effect | Source in our sim |
|---|---|---|
| **Terrain** | high ground, ford defense, enclosed streets, mud | world gen geography; `strategic-flexibility-chokepoints.md` |
| **Morale/cohesion** | units break before they die — morale from leadership, recent fortune, food, pay, belief | `sim/community.rs` cohesion, psych stress |
| **Supply** | hungry armies disintegrate — logistics IS the fight | economy stocks, supply lines |
| **Leadership** | a captain's competence/charisma decides more than unit stats | villager traits (reuse psych model) |
| **Surprise/initiative** | ambush, night, weather | event system |
| **Numbers... sometimes** | mass matters only when other factors are near-parity — overruns happen on disorganization, not ratio | derived |

## The model

- A combat is a **cohesion contest**: two forces, each with morale, cohesion, supply, position modifiers; they engage in exchanges where *cohesion damage* outpaces casualties — the side that breaks first loses, often with modest kills.
- Unit *types* exist (levy spears, archers, men-at-arms, cavalry) but with **role** effects not counter-multipliers: archers stress cohesion at range, cavalry exploit breaks, spears hold ground. They compose, they don't counter.
- Outcomes feed the world: battle scars, veteran units, dead named villagers (lifecycle), looting, ransom, vendettas — war propagates into the society sim.

## Non-battle combat

- Sieges as *starvation races* (supply vs walls) — mostly waiting, sickness, sorties; the wall is a multiplier on attacker supply burn
- Raids/skirmishes — the common case: stealing livestock, burning fields; economic warfare, not heroic combat
- Feuds/vendettas — interpersonal violence with family/memory (`relationship-modeling.md`)

## Related

- `simple-military-battle-system.md` (the implemented first pass)
- `strategic-flexibility-chokepoints.md`, `regional-ruler-personality.md`, `sim/military.rs`

## Rating

Fits the sim: war as an extension of society, not a minigame.
