# AI Player System for Beta Testing

**Category:** Design / Testing
**Source:** Backlog item (High) — "AI player system for beta testing."
**Date:** 2026-10-25

## Description

AI agents that *play* the game like a player would — not to test code (the component framework does that) but to test *the experience*: is it fun, is it readable, does a real player figure out the levers. The Tester agent specs (`wiki/agents/tester*.md`, `exploratory-tester-*.md`) implement this.

## AI player roles

| Role | Behavior |
|---|---|
| **Goal-oriented player** | given a goal ("grow to 100 villagers"), plays toward it with realistic input pacing |
| **Explorer** | pokes at systems, tries weird things, finds edge cases — the data-scientist-tester's play style |
| **Naive player** | no guide, figures it out — tests discoverability/onboarding |
| **Experienced player** | knows the systems, optimizes — finds balance problems, dominant strategies |
| **Griefer/chaos player** | tries to break it — feeds failure handling |

## What the AI player *is*

- A decision loop observing the world via the same surface a human player sees (the UI/telemetry), producing the same input stream — not a cheat harness; the AI plays *through* the interface
- Different player *profiles* are different policy functions over the same observation/action space — a "player model" spec per profile
- Heuristic baseline first (Simple/Heuristic AI Player item — already Completed); LLM-enhanced players where reasoning matters

## What it tests that unit tests can't

- **Playability** — can a player actually figure out how to do things? (the "Fun" rubric, `fun-rubric-automated-testing.md`)
- **Balance** — does a reasonable strategy dominate everything? (the Economist tester)
- **Emergence** — does interesting stuff happen? (the Emergent Effects researcher)
- **Readability** — can the player tell *why* something happened? (the Credibility Analyst)
- **Bug surface** — the griefer finds what invariants miss

## Feedback loop

- Each AI player run produces: telemetry (`telemetry-kpi.md`), a play log, an evidence report — the artifact a Tester spec consumes
- The Fun Inspector scores runs against the rubric; the Gameplay Consultant judges design
- Failures flow into the task queue → the auto-fix pipeline

## Related

- `wiki/agents/tester*.md`, `fun-rubric-automated-testing.md`, `component-testing-framework.md`, `1000-turn-testing.md`, `telemetry-kpi.md`

## Rating

The actual "is it fun" instrument — the thing no amount of unit tests replaces.
