# Emergent Storytelling in Complex Simulations

**Category:** Design / Narrative Systems
**Source:** Backlog item (High) — "Dynamic events & grand strategy storytelling; emergent storytelling in complex simulations."
**Date:** 2026-10-25

## Description

How stories emerge from systems interacting, without authored scripts — the project thesis. With ~9 agent specs and systems covering psych, economics, illness, religion, community, climate, magic, the story is what *falls out* when systems collide under pressure.

## Theory

Emergent narrative = causally-connected system outputs that a player *reads* as a story. The ingredients:

- **Readable causality** — players can trace *why* something happened (villager starved → son joined bandits → raid). Opaque emergence reads as noise.
- **Narrative-shaped events** — events that cluster around people (protagonists) and have consequence chains (setup→escalation→payoff), not isolated incidents.
- **Conflict as engine** — scarcity, grievance, ambition, incompatible goals. Every system should produce *tension* a story can wrap around.
- **Persistance** — consequences persist in the world (Datomic history gives us this literally).

## The event architecture

- Events are *system state excursions* rendered as narrative: when a register (hunger, stress, legitimacy) crosses a threshold, an event instance is minted with **participants** (who), **causes** (why, linked to upstream state), **stakes** (what could change).
- Chains: events spawn follow-ups on a cooldown — famine → hoarding → price spike → riot → crackdown. Each link is a *system transition*, not a scripted branch. `sim/emergence.rs` is the substrate.
- "Small events, big impact" (see `small-events-big-impact.md`): the most memorable emergence is a butterfly — a dropped delivery that cascades.

## Anti-patterns

- **Narrator faking it** — don't narrate consequences that didn't happen in the model; the sim must have actually done it (players detect fake causality)
- **Event spam** — cap simultaneous event density; events need room to be noticed
- **Disconnected incidents** — an event with no antecedent and no consequence is content, not story

## Implementation notes

- Reuse the register model from `machinations-resource-flow.md`: threshold crossings mint events; event metadata links causally to state diffs in telemetry/history.
- `agent_kb` gives events a *memory*: stories recur (a dynasty's fall echoes) — stories need reference back.
- The storyteller surface: chronicle view, village gossip (dialog), epilogue generation (see `dynamic-magic-story-text.md`, `playthrough-video-generation.md`).

## Rating

The project's crown-jewel concept — every other entry feeds this one.
