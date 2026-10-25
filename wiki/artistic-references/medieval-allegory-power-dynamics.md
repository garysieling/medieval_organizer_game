# Medieval Fantasy Allegory — Power Dynamics

**Type:** Narrative/Allegory Design
**Source:** Backlog item (High) — "community organizers vs fascists/billionaires in medieval fantasy setting."
**Date:** 2026-10-25

## Description

The game's political layer: medieval power structures as a *systemic allegory*, not a skin. The mapping is structural — nobles/crown/church/guilds instantiate the roles; organizers and extractive elites instantiate the dynamics. If the allegory is built into *mechanics* (who controls surplus, who controls narrative, how power reproduces itself), players discover it rather than being told it.

## The structural mapping

| Medieval form | Dynamic it carries |
|---|---|
| **Crown/nobles** | extraction by birthright + force monopoly: rent, taxes, corvée labor, enclosure |
| **Church** | ideological production + welfare monopoly: meaning, legitimacy, charity as pacification vs genuine mutual aid |
| **Guilds** | labor organization with dual nature: mutual protection AND exclusion/rent-seeking |
| **Merchants/patricians** | new money displacing old blood — capital vs lineage tension |
| **Peasant communes/villages** | where organizing happens: commons, custom, solidarity — and where it's crushed |

## Allegory through mechanics (not dialog)

- **Extraction visibility:** the economy sim already tracks surplus — a ruler's extraction rate is *legible in the model*; players see who eats and who doesn't. The allegory lives in the numbers.
- **Legitimacy as a resource:** rulers need consent OR force — church/belief systems (`sim/religion.rs`, `sim/fundamentalism.rs`) produce legitimacy; when belief wanes, extraction gets more violent or more negotiated.
- **Organizing mechanics are already designed:** `sim/community.rs` models 1-1s, house meetings, influence — the organizer's toolkit is the same toolkit that historically built unions/communes.
- **Counter-power emergence:** grievance + organization + opportunity → collective action — the sim should be capable of producing a commons movement or a crackdown without scripting either.

## Tone guardrails

- Satire through *systems*, not named caricatures — no literal billionaires in hats; the structure carries it (Bioshock's lesson: the ideology is the antagonist, not a villain's speech).
- Let sympathetic nobles and flawed organizers exist — the allegory is about *incentives*, not team jerseys. A benevolent lord who can't buck extraction pressure is a better story than a cartoon tyrant.

## Related

- `wiki/literature-reviews/sprint-3-community-organizing.md` (organizing mechanics)
- `sim/fundamentalism.rs`, `sim/community.rs`, `game/factions.rs`
- `regional-ruler-personality.md` (who wields the extraction)
