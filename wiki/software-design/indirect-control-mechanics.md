# Indirect Control & God-Game Mechanics

**Category:** Design / Player Interface
**Source:** Backlog item (High) — "Indirect control & god-game mechanics: influence vs command, player agency."
**Date:** 2026-10-25

## Description

The core player-interface philosophy: the player *influences* villagers — never clicks "move here" on a unit. Villagers are agents with their own needs/traits/relationships; the player shapes the *conditions* under which they choose. This is what makes the whole psychology/simulation stack matter.

## The influence palette (mechanics, not menus)

- **Infrastructure as suggestion** — build a bakery, villagers choose to bake; build a shrine, people pray. You shape the *option set*, not the choice.
- **Policy/law as constraints** — contracts (`game/contracts.rs`), taxes, permitted trades. Laws don't command, they price or forbid.
- **Resource provisioning** — stockpile flour and bakers appear; fund a militia and defenders organize.
- **Land designation** — zone commons vs fields vs market; villagers work what's available.
- **Patronage & appointments** — name a steward, a priest, a guildmaster — the *person* then governs by their traits.
- **Information manipulation** — signals villagers observe (flags, bells, festivals, warnings) — you control what they *believe*, at best.

## Design rules

- **Every command is a request to a simulated person** — a "build this" order becomes a task the villager may resist (skills, mood, loyalty all weigh). Failure modes are *characterful* — a resentful peasant builds slowly and badly.
- **Never expose direct unit orders** — no "select peasant → right-click." The UI never lets you *be* the villager's hands.
- **Agency asymmetry is the difficulty dial** — the more you control indirectly, the more the game's psych/community layers decide outcomes. Depth *is* the loss of control.
- **Accountable intermediaries** — appoint a bad steward → the village rots and it's *your* fault you named him, not the steward AI's. Responsibility reads through.

## Why this matters

Indirect control is what converts a city-builder into a *society sim*: the emergent stories (see `emergent-storytelling.md`) only exist because villagers have real agency the player navigates around. It also grounds the allegory layer (`medieval-allegory-power-dynamics.md`) — the player *experiences* the limits of command, which is the point.

## Related

- `player-agency-victoria-model.md` (the hybrid middle-ground)
- `single-character-indirect-control.md`, `delegated-agency-characters.md`
- `emergent-storytelling.md`, `psych-sociological-models.md`

## Rating

The load-bearing design choice — every system is downstream of it.
