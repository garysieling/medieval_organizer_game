# Skill Improvement with Degradation

**Category:** System Design / Simulation
**Source:** Backlog item — skills improve with use, decay without it.

## Description

Skill as a *flowing stock*, not a level: practice fills it, disuse drains it. This is more realistic than XP levels (a level-10 swordsman who hasn't swung in a decade is level-10 in name only) and produces useful simulation behavior: masters who retire decline, apprentices who grind rise, specialists vs generalists emerge naturally.

## Model

```
skill_level += practice_rate(practice_intensity, teaching?) * dt
            − decay_rate(skill_level, trait_modifiers) * dt
```

- **Practice rate** depends on intensity (hours worked at the craft), difficulty stretch (working slightly above level improves faster — deliberate-practice model), and teachers (a master's presence boosts the rate for apprentices — teaching is a skill economy).
- **Decay rate** is proportional-ish: high skills decay faster in absolute terms but may have a floor (muscle memory — riding a bike). Rust never takes a master to zero.
- **Per-domain splits:** decay/practice rates differ per skill family (physical crafts decay slower than academic knowledge; social skills decay on relationship inactivity — couple to relationships).

## Emergent hooks

- **Career arcs:** a smith who retires to farm slowly loses smithing; an injury forces re-skilling (couple to illness/lifecycle).
- **Guild value:** guilds aren't just economy — they're *practice pools*; being near other practitioners slows decay (ambient learning), which is a real reason guilds existed (`game/guilds.rs`).
- **Succession problem:** when a master dies, village skill level drops unless apprenticeship ran — a legible story the sim writes for free.
- **Specialization pressure:** one character can't keep N skills maintained — soft constraint on jack-of-all-trades without hard caps.

## Anti-grind guardrails

- Improvement requires *challenge-matched* work (no skill gain grinding trivial tasks — the classic exploit).
- Decay is slow enough that it reads as drift, not a timer anxiety mechanic.
- Skills plateau: diminishing returns near mastery so skill doesn't compound forever.

## Implementation notes

- Store skills as floats on the character component; update in a low-frequency system (per-day tick, not per-turn) — cheap.
- Teaching coupling: when a high-skill agent works adjacent to low-skill agents on the same recipe, boost their practice_rate — emergent mentorship.
- Evidence hook: run N years, check a retired master's skill drifts down and an apprentice's drifts up — one-line scenario for the test framework.

## Rating

Small system, outsized realism — and the enabler for lifecycle/career arcs across the whole sim.
