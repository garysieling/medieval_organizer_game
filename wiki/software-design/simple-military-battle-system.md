# Simple Military / Battle System

**Category:** System Design / Combat
**Source:** Backlog item — simple military/battle system. Implemented stub: `game2-core/src/game/military.rs`. Companion: `non-rps-combat.md` (the deeper combat model this stays compatible with).

## Description

War at the *campaign* level, not the tactical level: raising forces, readiness, a small set of engagement outcomes — enough for military pressure to be a real force in the world without becoming an RTS. Keeps the game's center of gravity on the society sim; battles are *events in the world*, not a second game.

## Scope discipline (what "simple" means)

- **Forces are pools, not units:** a settlement contributes levies/professionals to a force — quantity, quality, morale, supply. No individual soldier pathing.
- **Engagements are resolved, not played:** battle = f(force composition, leadership, terrain, supply, morale, luck) producing outcome + losses + story events. The *decisions* are all in preparation (who you raise, when you fight, where you meet them).
- **Aftermath is the content:** loot, casualties feeding lifecycle/grief, wounded→illness sim, prisoners/ransom (contracts/legal), territory shifts. The sim eats the consequences; that's the point.

## Components

- **Readiness:** levies vs retained professionals — raising peasants costs harvest labor (economy coupling); professionals cost upkeep year-round (the classic medieval tradeoff).
- **Terrain modifier:** uses `game/terrain.rs` — chokepoints/high ground matter (couples to `strategic-flexibility-chokepoints.md`).
- **Morale/readiness compartment:** force morale is a compartment (QSP pattern) — supply, leadership, and recent outcomes shift it; a demoralized army loses before the battle.
- **Siege:** simplified to attrition-vs-walls — supplies inside vs patience outside; starvation does most of the work (realistic and simple).

## Couplings (where battles feed the sim)

- **Economy:** levies pull labor at the worst time (harvest); war debt drives taxation → faction/unrest.
- **Lifecycle/psych:** casualties = grief events at scale; veterans = trauma compartment (theory-of-mind coupling).
- **Faction:** victories/losses move ruler legitimacy — war outcome feeds the power-dynamics allegory (`medieval-allegory-power-dynamics.md`).
- **Community organizing:** an unpopular war is an organizing substrate — recruitment resistance, desertion, morale collapse as emergent outcomes, not scripted events.

## Non-goals

- No tactical/unit-level combat (that's `non-rps-combat.md`'s deeper layer — this system remains the *resolution* interface even if a deeper model lands later).
- No real-time orders — the player commits, the sim resolves.

## Implementation notes

- `game/military.rs` holds forces/engagement resolution; outcomes emit world events (the sim's narrative feed) and telemetry for the testers.
- Evidence: a resolved battle should produce a readable aftermath record — who died, what changed hands — that a Data-Scientist tester can verify.

## Rating

Right-sized: war is a pressure on the sim, not a mode switch away from it.
