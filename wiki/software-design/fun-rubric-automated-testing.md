# "Fun" Rubric & Automated Playability Testing

**Category:** Design / Testing
**Source:** Backlog item (High) — "Fun rubric & automated playability testing."
**Date:** 2026-10-25

## Description

How to *measure* fun mechanically — the rubric the Tester agents score runs against. Fun is notoriously unmeasurable, but its *correlates* are: engagement signals, pacing, variety, surprise, recovery arcs, legibility.

## The rubric (scored per run)

| Dimension | Signal | How it's measured |
|---|---|---|
| **Engagement** | would a player keep playing | action density, decision points per hour, idle dead-zones |
| **Pacing** | tension/release rhythm | event cadence, pressure cycles, quiet-period length |
| **Variety** | not the same thing every run | entropy over system events, distinct storylines |
| **Surprise** | emergent moments | events with causal depth > 1 (see `emergent-storytelling.md`) |
| **Agency** | choices mattered | divergent outcomes across seeds from similar states |
| **Legibility** | the player can explain what happened | causal chains resolvable, telemetry traceable |
| **Stakes** | things mattered | meaningful state changes, real costs |
| **Comeback potential** | losing runs still playable | `comeback-events.md` triggers fired, recovery path existed |
| **Emotional texture** | the world felt alive | named villagers with arcs, relationship events, memorable moments |

## The scoring pipeline

- An AI player (`ai-player-beta-testing.md`) plays a run → produces telemetry + event log + action trace
- A scorer (LLM or heuristic) rates the run on the rubric — partially mechanical (event counts, pacing curves), partially judgment (was *that* fun?)
- The rubric output feeds the task queue — "low variety" → investigate event diversity

## What a rubric can't measure

- The rubric is a *proxy* — real fun is validated by humans eventually; the rubric's job is to catch the obviously-broken (dead zones, no variety, no stakes), not certify delight
- Don't overfit the rubric — a game optimized to the score is a different (and worse) game; the rubric is a screen, not a target

## Ties to agent specs

- `wiki/agents/tester*.md`, `exploratory-tester-*.md`, `fun-inspector.md` — the agents that consume this rubric
- `component-testing-framework.md` — the mechanical half (the rubric is the "is it fun" half of "does it work + is it fun")

## Rating

The hardest test to automate — treat as a heuristic screen, not a truth metric.
