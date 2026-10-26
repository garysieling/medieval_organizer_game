# Lit Review - Sprint 7: Roads That Strengthen With Use (Trail Formation)

**Date:** 2026-10-25
**Domain:** Complex Systems / Physics of Collective Motion / Landscape History
**Backlog items:** Roads (strengthen with use, fade without; stronger = faster; pathfinding feedback loop)
**Sources:** Helbing, Schweitzer, Keltsch & Molnár "Active Walker Model for the Formation of Human and Animal Trail Systems" (Phys. Rev. E 56, 1997); Edelstein-Keshet et al. "Trail following in ants: individual properties determine population behaviour" (Behav. Ecol. 1995); Wikipedia "Desire path"; Historic England *Pre-industrial Roads, Trackways and Canals* (2018); "Pavement management processes from medieval Europe" (CRC 2020)

## Sources

### 1. Active Walker Model (Helbing et al. 1997)
- Walkers **change the environment** (footprints/trampling) and the changed environment **steers future walkers**, a non-linear feedback loop
- Three parts: an **equation of motion** (steer to destination, attracted by nearby trails), a **ground-change equation** (trampling builds the trail, vegetation regrows/decays), an **orientation relation** (how far ahead walkers "see" trails)
- Frequently used trails get more developed and attractive; **rarely used trails vanish**; large detours get **shortcut**; new destinations join the existing network
- Pedestrians produce **minimal detour systems** (a compromise between direct routes and shared trails); ants produce dendritic (tree-like) systems
- Matches empirical trail systems; urban planners have used it to design paths people actually use
- **Relevance:** CRITICAL - this *is* the backlog item, with a published, simple model

Simplified model (paraphrasing the paper's structure):
```
G(x,t)        ground comfort / trail strength at tile x
dG/dt = (G0 - G)/T  +  I * (1 - G/Gmax) * footfall(x)
          decay back to       reinforcement, saturating
          natural state       at Gmax
walker heading = direction to destination  +  gradient toward strong trails nearby
```

### 2. Ant Trail Following (Edelstein-Keshet et al. 1995)
- Same feedback structure: pheromone is **laid** and **evaporates**; ants are recruited to trails and also **lose** them
- Order emerges only when **reinforcement and decay balance**; the paper quantifies the parameter regime
- **Relevance:** HIGH - tuning guide: too little decay and everything becomes road; too much and nothing persists

### 3. Desire Paths (Wikipedia, with trampling studies)
- Unplanned trails formed by erosion; usually the **shortest/easiest route** where official paths detour or don't exist
- Once visible, traffic follows them, so trampling erodes groundcover further and the path becomes preferred
- Impacts on vegetation happen **fast with initial use**; recovery is slow
- Some became major streets (Broadway is often cited as following a Wecquaesgeek trail)
- **Relevance:** HIGH - shows the full life cycle from desire path to trail to formal road

### 4. Pre-industrial Roads and Trackways (Historic England 2018) + Medieval Pavement Management
- **Trackway** = route marked by passing traffic over time; **road** = deliberately engineered route
- Medieval roads were mostly **maintained by obligation, not funding** (manorial/parish duty, tolls, charitable bequests for bridges); the chronic problem was **no steady funding**
- Heavy use without maintenance **sinks** routes: hollow ways (holloways) worn deep into soft ground
- **Relevance:** HIGH - two road classes (emergent trackway vs built road) and maintenance as a political/economic problem

## Mechanics

- **Per-tile `trail_strength`** (0 = wild, ~0.3 trackway, ~0.7 well-worn, 1.0 engineered road)
- **Update each tick:** reinforce by footfall (saturating), decay toward the terrain's natural value (forest regrows faster than heath)
- **Movement cost** = terrain cost × (1 − k·trail_strength), so strong trails are faster
- **Pathfinding** uses movement cost, so agents prefer existing trails, which reinforces them (the requested feedback loop). A small "directness" weight preserves shortcut formation (Helbing's minimal-detour behavior)
- **Engineered roads** have a high floor (decay slows to near zero) but need **maintenance labor/funds**; unmaintained roads degrade toward trackway
- **Overuse on soft ground** → holloway / mud (seasonal: wet season multiplies wear)

## Design Proposals

1. **Emergent road layer** driven by agent footfall, rendered as a darkness/width overlay on the map (fits the dynamic map evolution doc)
2. **Tuning knobs** from the ant paper: `reinforce_rate`, `decay_rate`, `saturation`, `visibility_radius`; expose them in the debug UI to find the "order" regime
3. **Upgrade path:** trackway → (labor + stone) → road. Who pays (lord via corvée, guild via tolls, parish duty) is a governance choice that feeds the taxes/feudalism item
4. **Desire-path politics:** planned roads that ignore desire lines get bypassed; a ruler who builds where people already walk gets more value per coin
5. **Trade feedback:** strong roads lower trade costs between towns, so towns on busy routes grow, which raises traffic further (rich-get-richer; an abandoned route fades and so does its village)
6. **Seasonal/weather coupling:** rain + traffic → mud → slower → traffic diverts → new parallel trails (braided paths)

## Rating
**Critical** - a well-studied, simple, published model that directly implements the requested feature, and it generates emergent geography, trade, and governance stories.
