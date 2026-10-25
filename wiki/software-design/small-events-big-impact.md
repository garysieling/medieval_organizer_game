# Small Events, Big Impact

**Category:** Design / Narrative Mechanics
**Source:** Backlog item (High) — "Small events, big impact: cascading consequences."
**Date:** 2026-10-25

## Description

The butterfly-effect design: small, mundane occurrences that cascade into major consequences — the mechanic that makes the world feel *alive* rather than scripted. A dropped delivery, a chance meeting, a petty insult becomes the seed of a feud, a famine, a romance, a revolution.

## Why it matters

- Emergent stories (`emergent-storytelling.md`) are built out of these — the memorable narrative is rarely the big scripted event; it's "and then the baker's son was late with the flour, so the miller's daughter waited, and they met"
- It rewards *attention* — players who notice small things anticipate cascades; it makes the sim feel like a real place where details matter
- It's the *comedy/drama source* — the most human-feeling events in a sim are the petty ones

## The cascade mechanism

- **Threshold event seeds:** any system state excursion can mint an event; *small* events are low-magnitude excursions (a missed delivery, a stumble, an overheard remark) that carry *consequence potential* — the event isn't the incident, it's the *seed*
- **Consequence propagation:** the event modifies state (a relationship edge, a resource stock, a belief) → downstream systems react → a later, bigger event mints → the chain continues until it dissipates or explodes
- **Causal memory:** chains link through telemetry/history — "this riot traces back to a market-day insult three weeks ago" is *retrievable*, which is what makes it feel emergent rather than random

## Example cascades

- Cart breaks down → flour arrives late → bakery can't fill the lord's order → baker loses the contract → can't pay the apprentice → apprentice leaves → villagers talk → reputation shifts
- Two villagers bump at the well → exchange words → one is a hothead → feud starts → families align → village factionalizes
- A peddler spreads a rumor → hoarding triggers → price spike → poor family can't buy → one steals → caught → punished → community splits over the sentence

## Design rules

- **The small event is the hook, not the payoff** — the payoff is downstream; design the *propagation*, not the incident
- **Density control** — too many seeds = noise; events must cluster into stories, not spew (cap active chain depth)
- **The sim must actually do it** — a "small event" that doesn't propagate is just flavor text; the value is the real causal chain

## Related

- `emergent-storytelling.md`, `comeback-events.md`, `trolley-problem-mechanics.md`, `sim/emergence.rs`

## Rating

The soul of the sim — where emergent narrative actually lives.
