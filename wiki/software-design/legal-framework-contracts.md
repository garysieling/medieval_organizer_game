# Legal Framework & Contract Types

**Category:** Design / Social Systems
**Source:** Backlog item (High) — "Legal framework & contract types: feudal contracts, land tenure, obligations."
**Date:** 2026-10-25

## Description

How medieval obligations are modeled: law and contract as the substrate on which land, labor, and loyalty run. `game/contracts.rs` implements this — this entry is the design vocabulary behind it.

## The medieval legal order (the system to model)

Medieval law wasn't one code — it was *overlapping customary obligations*:

- **Feudal tenure** — land held in exchange for service: the core contract. Lord grants land (fief) → vassal owes military/counsel; subinfeudation chains downward
- **Manorial/customary tenure** — peasants hold strips in exchange for labor dues (week-work on the lord's demesne, boon-work at harvest), plus usage rights (commons, gleaning, wood rights)
- **Guild contracts** — apprenticeship (years of service for training), journeyman work, mastership conditions; guild regulates entry + standards
- **Ecclesiastical obligations** — tithe, Peter's pence, church dues, moral jurisdiction (church courts handle marriage/oaths/wills)
- **Trade contracts** — debts, partnerships, commenda (investor + traveling merchant), bottomry (maritime loans)
- **Marriage/inheritance** — the kinship contract: dowry/dower, inheritance custom (primogeniture vs partible), widow's rights

## The contract as a game object

A contract is a data structure: `parties`, `obligations` (what each owes, when), `duration`, `enforcement` (who adjudicates — lord's court, church, guild, custom), `breach` (what happens when broken — fine, forfeiture, feud, excommunication). `game/contracts.rs` holds the template registry.

## Design principles

- **Customary > codified** — most medieval law was unwritten custom; our contracts default to customary obligations, and *written* contracts are the innovation (literate, enforceable, transferable — the route to capitalism)
- **Obligations run both ways** — the feudal contract binds the lord too (protection, justice); the medieval world wasn't all top-down extraction, it was reciprocal (even when unequal)
- **Enforcement is local** — no state to call; breach is adjudicated by a lord's court, a guild, the church — or resolved by feud/reputation. Enforcement quality varies by region (`regional-ruler-personality.md`)
- **Contracts change the economy** — enclosure contracts convert commons → private fields (allegory engine, `medieval-allegory-power-dynamics.md`); commenda creates merchant capital

## Emergent scenarios

- Peasant accumulating enough to *buy out* of labor dues (commutation) → free peasantry emerges
- A weak lord unable to enforce → de facto peasant autonomy
- Written charters displacing custom → the "paper revolution" that births capitalism
- Contract cascade failures: a lord's debt defaults → vassals' tenures collapse

## Related

- `game/contracts.rs`, `game/factions.rs`, `sim/economics.rs`, `medieval-allegory-power-dynamics.md`, `village-to-kingdom-scale.md`, `sprint-3-medieval-economics.md`

## Rating

The load-bearing structure — feudal society is literally a web of obligations; modeling contracts IS modeling the society.
