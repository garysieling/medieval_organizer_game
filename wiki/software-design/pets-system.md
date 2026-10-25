# Pets System

**Category:** System Design / Simulation
**Source:** Backlog item (Medium) — "Research RimWorld and Stardew Valley pet systems. Bonding, utility, emotional impact." Implemented stub: `game2-core/src/game/pets.rs`.

## Description

Animals that live with characters — three roles in one: *bonded companion* (emotional mechanics), *working animal* (economy), and *story generator* (loss, rescue, conflict over pets). The RimWorld/Stardew reference points both matter: RimWorld's bonded animals produce real grief when they die; Stardew's pets are mostly ambient affection.

## Research takeaways

- **RimWorld (bonded animals):** animals bond to specific colonists; bond = mood bonus while alive, real grief penalty on death (scaled by bond). Pets also have utility (hauling, guarding) and *nuzuzzler* moments (animals self-taming, bonded animals defending owners). The grief mechanic is the lesson — a pet death is one of RimWorld's best emergent story events.
- **Stardew Valley (ambient affection):** pet exists at the farm, gets petted, mostly vibes — the lesson is *low-friction warmth*: a pet doesn't need management depth to contribute mood.
- **Combined design:** pets should be meaningful but not another job queue — a bond tier (acquaintance → companion → bonded) on a relationship edge to an animal entity, and utility roles that emerge from species traits, not pet micromanagement.

## Model

- Pet = entity with species traits (dog: guard/hunt; cat: pest control/comfort; livestock-pet edge cases), needs (food, shelter, attention), and a bond edge to one or more characters.
- **Bond tiers** on the relationship edge (reuses `game/relationships.rs` structure — a pet is an edge with a narrower dimension set: warmth + dependency).
- **Utility from species:** guard dogs alert/deter, cats keep pests down (a real effect on food storage), herding dogs assist livestock work — utility is passive, not task assignment.
- **Emotional payload:** bonded pet presence = small steady mood/stress benefit (compartment outflow, `qsp-theory-of-mind.md`); pet death = grief event scaled by bond — couples to attachment system so a *child's* first pet loss is a real development event.
- **Ambience:** pets wander, greet owners, appear in the isometric board — they make the village feel alive cheaply.

## Emergent hooks

- **Adoption events:** strays arrive during scarcity/abundance patterns (climate + economy coupling); a starving dog at the granary is a tiny moral choice.
- **Pet conflicts:** bonded pets in disputes (who keeps the dog when characters feud) — relationship drama the sim writes.
- **Working animals vs pets:** the line blurs — a bonded sheepdog is both; the bond changes the cost of losing "a working asset" into "a family member."

## Implementation notes

- `game/pets.rs` holds species/needs/bonds; the bond edge integrates with `game/relationships.rs` so pets appear in the social graph (`ui/graph.rs`) — delightful and cheap.
- Death/grief hooks go through the lifecycle and psychology sims already in place.

## Rating

Medium priority, high warmth-per-line: reuse the relationship/psychology machinery, get an emotionally resonant system.
