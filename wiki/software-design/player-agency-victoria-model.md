# Player Agency vs Simulation — the Victoria Model

**Category:** Design / Analysis
**Source:** Backlog item (Medium) — "Player agency vs simulation analysis (Victoria case study)."
**Date:** 2026-10-25

## Description

Case study: Paradox's Victoria series is the closest existing model to what game2 attempts — a *deeply simulated society* where the player is an influential spirit of the nation, not its puppeteer. Analyzing what Victoria gets right/wrong grounds our design choices.

## The Victoria model

- **Pops as the sim:** the world is made of population units (pops) with class, culture, religion, politics, needs — they have *interests*, not orders. The player can *influence* (laws, taxes, buildings, wars) but cannot command a pop to do anything.
- **Player as the state, not the people:** you set laws, build institutions, declare wars — the *machinery* of power — but the pops respond on their own terms (they migrate, radicalize, organize, rebel).
- **Interest groups as the political layer:** landowners, industrialists, intelligentsia, armed forces, trade unions — each is a pop aggregation with a stance; politics is managing/empowering/suppressing them.
- **The economy runs itself:** goods flow through prices and production — the player *builds* capacity (buildings, trade routes) but the economy's behavior emerges from supply/demand/pops.

## What game2 shares

- Agents/populations with their own motivations — our villagers are pops with richer psychology (`psych-sociological-models.md`)
- Player as *influence on the system*, not commander — our `indirect-control-mechanics.md` is Victoria's philosophy zoomed to the village scale
- Politics as *group* dynamics, not leader stats — our `sim/community.rs`, `sim/fundamentalism.rs`, `factions.rs` do Victoria's interest-group work at village scale
- Laws/policies as the control surface — our contracts/laws (`legal-framework-contracts.md`)

## Where game2 diverges

- **Zoom level:** Victoria is the *state* scale (national politics); game2 is the *village* scale — the same model at a different magnification. Individual villagers are visible; pops are aggregates.
- **Person-level richness:** Victoria's pops are demographics; our villagers are *people* (traits, relationships, life histories) — far richer per-entity, far fewer entities
- **Time scale:** Victoria runs decades of national time; game2 runs years of human time — different granularity of consequence

## Lessons for our design

- **The "watch it run" appeal is real** — the Victoria player finds joy in *watching the society breathe*, not in control; game2's job is to make a *village* that compelling to watch
- **Interest groups > character stats** — the political meat is in *groups* with interests; our factions/community sims map to this
- **Let the sim be the protagonist** — Victoria's best moments are emergent (a migration wave, a crisis, a revolution); the story is *the society's*, not the player's

## Rating

The closest design precedent — the north star for "simulated society" done right.
