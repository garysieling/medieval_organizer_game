# Age-Based Self-Interest Economy

**Category:** Design / Economics
**Source:** Backlog item (High) — "Age-based self-interest economy: generational differences."
**Date:** 2026-10-25

## Description

How villagers' economic behavior shifts across the lifecycle — children, young adults, householders, elders have different goals, consumption, and political interests. A village's *age structure* is itself an economic and political variable.

## The age-graded interest model

| Life stage | Economic behavior | Political interest |
|---|---|---|
| **Child** | consumption only; learns skills through work/play | none direct — objects of household strategy |
| **Youth (apprentice)** | earns little, invests in skill; mobility-seeking | land access, marriage prospects — restless, recruitable |
| **Householder (prime)** | peak production; accumulates for children; risk-averse on fundamentals, risk-taking on surplus | stability, taxes, children's inheritance — the conservative center |
| **Elder** | declining production; holds land/status; needs care | legacy, honor, keeping children near, customary order |

## Generational dynamics

- **Inheritance is the hinge** — who gets the land/workshop decides whether children stay or migrate; primogeniture → young siblings migrate/organize; partible inheritance → fragmentation
- **Old vs young structural conflict** — elders hold assets, youth hold labor; the wage/land access balance IS generational politics. When youth can't start households, you get migration, banditry, or radical movements — the political tinder of medieval society (see `medieval-allegory-power-dynamics.md`)
- **Cohort effects** — a famine cohort is shaped forever (smaller, harder, thriftier); a golden-age cohort expects ease — cohort = history carried by people
- **Care economy** — elders consume care; young families produce it; societies that can't organize elder care generate family-level tragedies

## Mechanic mapping

- Lifecycle stage (`lifecycle-simulation.md`) → consumption profile + production capacity + risk/interest profile
- Household unit pools wealth and allocates among members — the *household* is often the real economic agent, not the individual
- Elder care duty is a real obligation system (villagers owe kin; absconding = reputation hit)
- Age structure → polity temperament: an aging village is conservative/risk-averse; a youth-heavy frontier village is volatile/opportunistic

## Tie-ins

- `lifecycle-simulation.md` (stages), `relationship-modeling.md` (kinship duty), `sim/economics.rs` (consumption/production profiles), `village-to-kingdom-scale.md` (demographic structure of the polity)

## Rating

A quietly powerful axis — turns population age into a real mechanic, and makes inheritance the most consequential "tech" in the game.
