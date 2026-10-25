# Character Sheet UI Design

**Type:** UI Reference
**Source:** Backlog item (High) — "character sheet UI: villager information display." Implemented: `ui/character_sheet.rs`.
**Date:** 2026-10-25

## Description

The villager detail view — where all the depth systems surface to the player. A character sheet for a *person*, not a stat block: identity, traits, skills, relationships, history, needs, obligations.

## What a villager sheet shows

| Section | Source system |
|---|---|
| **Identity** | name, age/lifecycle stage, sex, portrait (`lifecycle-simulation.md`) |
| **Vital state** | health, illness stage (`deep-illness-simulation.md`), stress/fatigue |
| **Traits** | Big-5 disposition + moral axes (`psych-sociological-models.md`, `dantes-divine-comedy.md`) |
| **Needs** | Maslow-style unmet needs driving behavior |
| **Skills** | craft skills with proficiency tiers (`skill-improvement-degradation.md`) |
| **Relationships** | the social graph: kin, friends, rivals, employer, apprentice — with trust/warmth scores (`relationship-modeling.md`) |
| **Obligations/contracts** | tenancy, apprenticeship, debts, guild membership, feudal dues (`legal-framework-contracts.md`) |
| **Property/economy** | what they own, earn, consume |
| **Beliefs** | religious stance, faction sympathies (`sim/religion.rs`, `sim/fundamentalism.rs`) |
| **History** | a chronicle of their life events (births, marriages, feuds, disasters survived) — the narrative of a person |

## Design principles

- **Person, not stats** — the sheet should read as "who is this person?" not a spreadsheet; narrative framing over stat dump
- **The relationships are the sheet** — the social graph view is the most game2-specific thing: show *who they are to others*
- **History is the payoff** — a villager who's survived a plague, lost a spouse, apprenticed badly, and now runs the bakery — the sheet should *tell* that story
- **Actions from the sheet** — this is where the single-character indirect controls live (`single-character-indirect-control.md`): suggest, appoint, gift, introduce — from their sheet

## Implementation

- `ui/character_sheet.rs` — implemented; renders villager entity data (already producing renders)
- Portrait: procedural pixel-art face from traits (age/gender/features) or generated portrait (`art-asset-library.md`)
- The sheet is the *deep* view — the overworld is the wide view; the sheet is where the player falls into the sim's depth

## Rating

The sheet IS the game in microcosm — the interface to the society.
