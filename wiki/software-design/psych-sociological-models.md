# Psychological & Sociological Models in Game Design

**Category:** Design / Research Synthesis
**Source:** Backlog item (High) — "Psychological & sociological models: Maslow, attachment, social networks, emergent behavior."
**Date:** 2026-10-25

## Description

Synthesis entry tying together the psychology/economics/community literature into a model stack for villager & settlement simulation — where each theory lands as a mechanic.

## The model stack

| Layer | Theory | Implementation |
|---|---|---|
| **Needs** | Maslow hierarchy as needs stack: physiological → safety → belonging → esteem → self-actualization; lower unmet needs dominate behavior | need vector per villager; unmet low needs gate higher motivations (`sim/psychology.rs`) |
| **Attachment** | Petters: proximity, safe haven, separation distress, secure base | caregiver/bond edges; covered in `sprint-1-psychology-economics.md` + `relationship-modeling.md` |
| **Dispositions** | Big-5 traits + moral disposition (Dante's vice/virtue axes) | trait vector on villager entity |
| **Stress/load** | QSP-style compartments: stress, grief, burnout accumulate/decay | `sim/psychology.rs` + `qsp-theory-of-mind.md` |
| **Social network** | multi-dim relationship edges (warmth/trust/respect/history) | `relationship-modeling.md` |
| **Collective** | contagion, cohesion, radicalization, organizing | `sim/community.rs`, `sim/fundamentalism.rs`, `sim/emergence.rs` |

## Key decisions

- **Needs gate motivation, not happiness** — a starving villager doesn't feel "unhappy," they *redirect activity* toward food. Needs modulate the utility function, not a mood meter.
- **Traits are stable, states are dynamic** — Big-5 + moral axes change slowly (life events, age); stress/mood/relationships change fast. Same fast-state/slow-disposition split as the illness model.
- **Social structure is a graph, not a score** — community cohesion, rumor spread, contagion all run over the relationship graph, not aggregates.
- **Emergence is validation** — the model is working when a *believable* individual behavior aggregate produces a *surprising* collective outcome (a mob, a schism, a migration).

## Model boundaries (avoid over-modeling)

- Don't simulate consciousness — simulate *decision-relevant* psych (motivation, attachment, stress, beliefs-as-priors). Theory of mind only where NPCs model each other (see `qsp-theory-of-mind.md`).
- Keep every psych quantity visible to mechanics — a psych variable that changes no decision is dead weight.

## Related

- `sprint-1-psychology-economics.md`, `sprint-3-community-organizing.md`, `sprint-3-religion-emergence.md`
- `relationship-modeling.md`, `qsp-theory-of-mind.md`, `sim/psychology.rs`, `sim/community.rs`

## Rating

The unifying frame — psych is the soil every social sim grows from.
