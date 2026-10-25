# Single Character Indirect Control

**Category:** Design / Player Interface
**Source:** Backlog item (High) — "Single character indirect control: Sims-like influence."
**Date:** 2026-10-25

## Description

The zoomed-in version of indirect control: the player focuses on *one* villager and nudges their life — suggestions, opportunities, social maneuvers — without puppeting. Sims-like influence but with our psych/relationship machinery underneath; complementary to `indirect-control-mechanics.md` (the zoomed-out god-game version).

## The influence verbs on one character

- **Suggest/encourage** — "you should apprentice with the smith" (lands better if the smith already likes them; worse if resentful)
- **Provide opportunities** — buy the apprenticeship, gift a tool, sponsor a journey — lowering the cost of a path makes it likely, not mandatory
- **Social engineering** — introduce two villagers, invite someone to a feast, put a rival in their path — you steer *who they meet*
- **Grant access** — open the workshop to them, lend the good plow, write them a letter of recommendation
- **Environment shaping** — where they live, what chores they get, who they work beside — subtle steering through circumstances

## What the character does with it

The villager evaluates suggestions against their own **goals + traits + relationships + current state**: a proud youth may reject a patronizing suggestion; a grieving widow may not respond; an ambitious apprentice may seize the opportunity and surpass the plan. The suggestion is *input to their decision*, not a command — see `psych-sociological-models.md`.

## Design rules

- **Relationship weight modulates influence** — you can't suggest effectively to a stranger; influence accrues through repeated, respected interactions (fits `relationship-modeling.md` — suggestion success is a function of the edge's trust + warmth + history)
- **Suggestions have memory** — a villager remembers you pushed them into a bad apprenticeship; your reputation in that relationship updates
- **Failure is legible** — when a suggestion fails, the reason is visible (they want something else, they don't trust you, they're too proud) — failure teaches the player about the character, not just "declined"
- **One at a time** — the mode is intentionally narrow; focusing on one villager is what creates the intimacy/story

## Emergent play

- Matchmaking a village
- Grooming a successor for a steward
- Rescuing a youth from a bad path (or failing to)
- Playing fixer/patron — the medieval-godfather fantasy, mechanically grounded

## Related

- `indirect-control-mechanics.md` (the broad version), `delegated-agency-characters.md`, `relationship-modeling.md`, `psych-sociological-models.md`, `character-sheet-ui.md`

## Rating

The emotional entry point — where the society sim becomes personal.
