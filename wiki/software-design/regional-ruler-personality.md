# Regional Ruler Personality Variation

**Category:** Design / World Systems
**Source:** Backlog item (High) — "how to make game feel different across regions."
**Date:** 2026-10-25

## Description

How regions/rulers vary so crossing a border *feels* different — different laws, aesthetics, priorities, and how each interacts with villagers. "Left or center rulers vs right wing" in the backlog's terms maps onto a richer continuum: disposition × policy × culture, not a political spectrum toggle.

## Model

Three independent layers per region:

| Layer | What's in it | Where it lives |
|---|---|---|
| **Ruler personality** | openness, conscientiousness, extraversion, agreeableness, neuroticism (reuse villager psych traits) + governing traits: ambition, cruelty, piety, pragmatism | ruler entity, `sim/psychology.rs` machinery |
| **Policy stance** | extraction rate, tolls, laws (contracts.rs), religious posture (tolerant↔orthodox), militia vs professional force, commons policy | `game/factions.rs` + contracts |
| **Regional culture** | trade-vs-agrarian weight, festival calendar, architectural style, staple goods, relationship norms | world gen + `regional-architectural-variety.md` |

## Why personality ≠ policy ≠ culture

- A *kind* ruler can preside over *harsh* law (captive of the machine); a *crude* ruler can run a *lenient* fief (doesn't care). Decoupling the layers produces the interesting cases — this is the anti-cliché move.
- Culture drifts slower than rulers: a brutal regime can occupy a warm mercantile town — the friction IS the story.

## Felt differences for the player

- **Prices & goods:** staple goods differ → market.rs produces different economies per region (already grounded in `sprint-3-medieval-economics.md` regional staples)
- **Contracts:** what's legal differs — enclosure banned in one region, encouraged next door (see `legal-framework-contracts.md`)
- **Faces:** villagers migrating across borders carry different psych baselines and relationship norms
- **Look:** distinct regional palettes/silhouettes (artistic-references)
- **Saints/feast days:** different calendar rhythms → different event cadence

## Interaction effects (the good stuff)

- Pious ruler + heretical village → pressure cooker (feeds `sim/fundamentalism.rs`)
- Pragmatic ruler + border region → tolerance as policy, not virtue
- High-extraction ruler + organized communes → the allegory engine (`medieval-allegory-power-dynamics.md`) ignites
- Personality drifts under pressure: wars make rulers crueler; abundance softens them

## Implementation notes

- Ruler is a specialized villager-like entity — same psych machinery, plus office traits; `factions.rs` maps region→stance; policies are contract templates keyed by region
- World gen assigns regions a cultural template + ruler draw; subsequent rulers drawn from succession logic (lifecycle → heir)

## Rating

This is the feature that makes a *map* feel like a *world*. High leverage on existing systems.
