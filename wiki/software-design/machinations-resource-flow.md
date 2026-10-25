# Machinations-Style Resource Flow Design

**Category:** Design / Systems
**Source:** Backlog item (High) — "design system to visualize/balance resource flows."
**Date:** 2026-10-25

## Description

Use Dormans' Machinations vocabulary (covered deeply in `sprint-1-game-design-simulation.md`) as the *design language* for the game's economies — pools, sources, drains, converters, registers — both for internal tuning and as the backend model behind any economy visualization.

## Core vocabulary (recap)

- **Pool** holds a resource quantity (granary stock, treasury, village morale)
- **Source** produces (farm output, taxes), **Drain** consumes (population eating, upkeep)
- **Converter** transforms resource→resource at a rate (flour→bread, ore+charcoal→iron)
- **Register** computes a *derived value* from pool states (hunger index from food vs mouths, legitimacy from extraction vs compliance)
- **Activators/triggers** gate flows (harvest season opens source, war opens drain)

## Mapping onto game2

| Machinations element | Our system |
|---|---|
| Good pools | `game/economy.rs` stocks, `sim/economics.rs` flows |
| Converters | `game/recipes.rs` (recipe tree already models inputs→outputs) |
| Registers | derived meters: hunger, stress, cohesion, legitimacy |
| Sources/drains | production tasks, lifecycle consumption, disasters |
| Actuation | policies/contracts open/close flows (enclosure, tolls, taxation) |

## Why it matters for us

- **Balance becomes visual before it's numerical** — an economy is a graph of flows; deadlock/starvation points are *visible* in the structure before a playtest finds them.
- **The deterministic sim is literally a Machinations engine** — a graph of pools and flows stepped in ticks; building economy design in this vocabulary keeps it analyzable.
- **Agent-legible:** an AI designer reads/edits a flow graph more reliably than a heap of rate constants — a machinations-DSL layer over economy systems is cheap and high-value.

## Recommended mechanics to extract

- Express economy config as a flow graph (pools + converters + registers) rather than scattered constants — data-driven economy tuning
- Registers for every pressure we model (hunger, stress, legitimacy, belief) — they're all pool-derived meters
- A visualization/debug view rendering the live flow graph — debugging economies by watching arrows, not diffs
- "Bottleneck = story" — the game's emergent economy events are places where flows stall or flood

## Related

- `wiki/literature-reviews/sprint-1-game-design-simulation.md` (Machinations primary)
- `game/economy.rs`, `game/recipes.rs`, `sim/economics.rs`
- `emergent-storytelling.md` (flow stalls → events)

## Rating

Conceptual backbone for the economy layer — treat the sim as an explicit flow graph.
