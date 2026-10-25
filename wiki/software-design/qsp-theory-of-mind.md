# QSP for Theory of Mind

**Category:** System Design / Simulation
**Source:** Backlog item — apply the QSP compartment pattern (`wiki/literature-reviews/sprint-3-qsp-game-systems.md`) to mind/mental-model simulation. Extends `deep-illness-simulation.md` and `relationship-modeling.md`.

## Description

QSP's trick — compartment + flow rates + intervention effects — applied to *mental state*: beliefs, moods, stress, models-of-others. A character's psychology becomes a set of compartments (stress reservoir, trust pool, belief strengths, attachment states) with inflows/outflows from events, and interventions (reassurance, ritual, conflict, rest) shifting the rates. This is how "psychological state" becomes simulatable instead of scripted.

## Why this framing

- **Mental states accumulate and drain** — exactly what compartments model. Stress isn't an event, it's a reservoir with inflow (threat, loss, conflict) and outflow (rest, ritual, social support, time).
- **Interventions have dose-response** — a kind word doesn't reset trust; it nudges a rate. QSP's intervention-shifts-rate pattern is literally how real support works.
- **Compartments interact** — chronic stress spills into attachment expression and relationship behavior; illness compartments (`sim/illness.rs`) spill into mental ones. The interaction network is where emergence lives.

## Candidate compartments (first model)

| Compartment | Inflows | Outflows | Interventions |
|---|---|---|---|
| **Stress** | threat, loss, conflict, crowding, hunger | rest, ritual, safety, social support | gatherings, sanctuary spaces, festivals |
| **Trust (per-agent or per-group)** | kept promises, fair dealing, honesty | betrayals, extraction, lies | contracts honored (`game/contracts.rs`), reparations |
| **Belief strength (per-narrative/ideology)** | sermons, echoed claims, confirming events | disconfirming events, counter-narrative | preaching, community norms, censorship (feeds `sim/fundamentalism.rs`) |
| **Attachment security** | warmth episodes, consistent care | anxiety episodes, neglect, abandonment | stable relationships, predictable caregiving (`sim/psychology.rs` already implements emergence here) |
| **Morale/hope** | wins, progress, meaning | setbacks, grind, hopelessness | festivals, achievable goals, novelty |

## Theory-of-mind layer (the interesting half)

Each agent also keeps a *model* of key others — the agent's guess of their compartments. That's theory of mind as data:

- Agents act on their *model* of another's stress/trust, not the real value — misreads produce realistic social friction ("she thought he was furious; he wasn't").
- Models update on observed behavior (a smile lowers assumed stress — possibly wrongly).
- Deception = deliberately managing others' models of your compartments (acting calm to keep your stress compartment hidden) — a real mechanic for intrigue, organizing, and court politics.

## Emergent hooks

- **Radicalization:** belief + stress compartments high + belonging outflow blocked → fundamentalism emergence conditions (the `sim/fundamentalism.rs` design target).
- **Burnout:** stress reservoir chronically near max → productivity collapse, health spillover.
- **Charisma as model-influence:** leaders shift *others'* belief/trust compartments — organizing is literally compartment-rate intervention at scale (couples to `sim/community.rs`).

## Implementation notes

- Reuse the `sim/illness.rs` compartment scaffold (same update math: rates, thresholds, interventions).
- Keep compartments sparse at first (3-4 per agent); the ToM layer is per-relationship, only for salient relationships (cost control).

## Rating

The highest-leverage Creative Filter port in the backlog — one QSP pattern, three domains (illness, mind, belief).
