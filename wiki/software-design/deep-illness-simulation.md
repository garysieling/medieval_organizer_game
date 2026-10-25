# Deep Illness Simulation (e.g., Cancer)

**Category:** System Design / Simulation
**Source:** Backlog item — deep illness simulation beyond generic "sick" states. Implemented stub: `game2-core/src/sim/illness.rs` (QSP compartments); see `wiki/literature-reviews/sprint-3-qsp-game-systems.md`.

## Description

Illness modeled as a multi-stage process with its own dynamics — onset, progression, treatment response, remission/decline — instead of a binary sick/healthy flag. Uses the QSP (Quantitative Systems Pharmacology) compartment pattern already adopted for this project: state compartments + flow rates + intervention effects.

## Why "deep" matters for gameplay

A binary sick flag produces exactly one story: "X is sick → treat → cured/dead." A staged model produces distinct, story-bearing trajectories:

- Early asymptomatic phase — illness exists but is invisible; detection depends on diagnostics (which tech must enable — see `technology-medical-options.md`)
- Progression stages with different symptoms, care needs, and outlooks
- Treatment response *curves* (responders / partial / refractory), not success/fail
- Remission with recurrence risk — an illness can *end* without being gone
- Chronic courses — lifelong management, not cure

Cancer is the canonical case (screening → stage I-IV → treatment options per stage → survival distributions → recurrence), but the same scaffold covers plague outbreaks, chronic conditions, and degenerative disease.

## Model (QSP compartment style)

```
Susceptible → Exposed(asymptomatic) → Stage 1..N → Treated/Remission → Recurrence → Deceased
                  ↑ detection gates here        ↑ treatment gates here
```

- Flows are rates modified by host factors (age, constitution), environment (nutrition, crowding), and tech (available diagnostics/treatments).
- Interventions shift rates per-stage: a treatment that works at Stage 1 may do nothing at Stage 3 — *timing* becomes a mechanic, not just access.
- Costs are multi-dimensional: supplies, healer time, patient productivity, family burden (feeds `game/lifecycle.rs` + relationships).

## Emergent hooks (what other systems consume)

- **Attachment/relationships:** serious illness reshapes attachment — caregiving bonds, anticipatory grief, family strain (couple to `sim/psychology.rs`).
- **Economy:** prolonged illness = labor loss + healer demand; medieval economics already has price formation to feel scarcity.
- **Religion:** mortal illness is a driver of religious seeking (couple to `sim/religion.rs`) — and of pseudo-medical demand (couple to `game/herbology.rs`, where remedies *mostly* don't work but sometimes help).
- **Medicine tech tree:** diagnostics unlock visibility; treatments shift rates; neither exists by default in a medieval sim — this *creates* the tech tree's value (see `technology-medical-options.md`).

## Design guardrails

- **Dignity-first:** illness is a simulation of *systems*, not a suffering generator. Show trajectory + care, not gratuitous grimness. (Keep the witchy-apothecary tone from herbology.)
- **Legibility through characters:** player learns of illness via symptoms/diagnosis on a character sheet — the compartment model stays underneath.
- **Not every illness is modeled** — keep 2-3 deep exemplars (a cancer-like progression, an infectious outbreak, a chronic condition); everything else uses the generic shallow path.

## Rating

High narrative density per system — illness is where psychology, economy, religion, and tech all intersect on one character.
