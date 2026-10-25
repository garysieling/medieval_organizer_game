# Faction Asymmetry in Strategy Games

**Category:** Design / Strategy Systems
**Source:** Backlog item (High) — "Faction asymmetry: unique abilities, asymmetric gameplay."
**Date:** 2026-10-25

## Description

How factions/regions differ so playing or facing each feels different — beyond palette swaps, into genuinely different *ways of operating*. In our medieval society-sim context, asymmetry comes from **structural differences in economy, social organization, belief, and geography**, not ability hotkeys.

## Asymmetry axes for game2

| Axis | Faction expressions | Mechanic carrier |
|---|---|---|
| **Economy** | agrarian communes vs trade republics vs extraction fiefdoms vs herding peoples | production profiles, contract types, staples (`sprint-3-medieval-economics.md`) |
| **Authority** | feudal lordship vs oligarch council vs ecclesiastical domain vs tribal assembly | governance model → how the player influences (patron vs lobbying vs piety) |
| **Belief** | orthodox vs reformist vs heterodox vs pagan-holdout | `sim/religion.rs` stances → legitimacy costs, festival calendar |
| **Social structure** | nuclear villages vs extended clans vs guild towns vs commons | relationship density, organizing propensity (`community.rs`) |
| **Geography-derived** | mountain holds (defense/toughness), ports (trade), frontier (raids) | world gen regional template |

## Design principles

- **Asymmetry through incentives, not powers** — a "trade faction" isn't given +20% trade; it's *positioned* so trade is what it does (ports, craft guilds, merchant elite). The asymmetry falls out of the structure — simulation-first.
- **Player-facing asymmetry = different problems** — each faction presents a different *set of frictions*, not a different set of buttons.
- **Mirror asymmetry** — a communal faction facing an extraction fief plays defense through solidarity; an extraction fief facing communal neighbors plays co-optation or force. Asymmetry is most interesting in *interaction*.
- **No "balance to parity"** — factions aren't supposed to have equal paths to the same outcome; a poor region's story is struggle, a rich region's is decay or corruption.

## Ties to existing systems

- Region templates from `world-generation-exploration.md` + `regional-ruler-personality.md` produce the faction profile
- `game/factions.rs` is the carrier; stances/policies flow into contracts, economy, religion
- Allegory layer (`medieval-allegory-power-dynamics.md`) gives factions *meaning* — the asymmetry isn't cosmetic, it's moral/political structure

## Rating

A different shape than StarCraft-style race design — ours is structural asymmetry, which suits a society sim better.
