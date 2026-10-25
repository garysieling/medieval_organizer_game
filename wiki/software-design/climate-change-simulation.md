# Novel Threat Systems — Climate Change Simulation ("You vs. Climate")

**Category:** System Design / Simulation
**Source:** Backlog item — "'you vs climate change' as a novel threat system." Implemented stub: `game2-core/src/sim/climate.rs` (seasons/drift).

## Description

An antagonist that isn't a faction: slow-moving, partially anthropogenic, mostly predictable in trend but stochastic in event. The design problem with climate as a threat in games is that it's *boring when legible* (a doom meter you feed resources into) and *unfair when invisible* (random punishments). This design makes climate legible as a *system*, not a meter.

## Research: how existing games handle it

- **Civ VI: Gathering Storm** — CO₂ → temperature bands → flooding/storms/droughts. Works mechanically (global collective number everyone feeds) but reads as a spreadsheet, not a threat — players optimize a meter, don't *feel* climate.
- **Frostpunk** — climate as the fixed hostile environment; temperature drives everything and the dread is *felt*, but it's scripted, not emergent.
- **Against the Storm** — recurring hostile weather windows (storm seasons) as a pacing device: predictable cadence, unpredictable specifics.
- **Dwarf Fortress** — no climate sim, but weather/biome realism teaches: legibility comes from *effects on things players care about*, not from abstract indices.

Takeaway: the threat lands when climate acts on the things the player owns (crops, rivers, buildings, people), on a cadence they can partially predict, with uncertainty in the specifics.

## Design: climate as layered forcing

1. **Trend layer (slow, semi-legible):** regional climate index (temp/water) drifts with driver pressure — deforestation, irrigation, industry, latitude. Players see *direction*, not precise trajectory.
2. **Season layer (cyclical, legible):** seasons modulate the trend; a bad season = trend + bad luck, not a random event. `sim/climate.rs` already models seasons+drift.
3. **Event layer (stochastic, telegraphed):** droughts, floods, storms, late frosts emerge when trend×season crosses thresholds; *forecast* (imperfect) precedes each — decisions matter because the forecast can be wrong.
4. **Adaptation layer (player response):** crop choice, storage, building tech, migration. Never a single "reduce CO₂" knob — climate is adapted to, not solved.

## Mechanics to extract

- Climate couples to *everything*: harvest yield, river navigability, fire risk (ties to `sim/emergence.rs` CA fire), disease, migration pressure.
- Regional asymmetry: same forcing, different local outcomes → terrain realism matters (see `realistic-natural-structures.md`).
- Intergenerational: the trend is slow enough that a ruler's choices land on their successor — feeds the lifecycle/legacy systems.
- Emergent conflict: drought upriver → downstream famine → migration → faction friction. The threat writes stories.

## "Novel" angle

Almost nobody makes the player *a community* adapting to climate rather than a state fixing it. Ours: village-scale, the threat is managed through collective adaptation (storage, norms, rituals — couples to religion/community systems). The satirical/allegory hook: rulers can deny, defer, or adapt — and the sim tracks which strategy their personality picked (ties to Regional Ruler Personality Variation).

## Implementation notes

- `sim/climate.rs` provides the drift/season base; extend with the threshold→event layer emitting world events (`events/climate_*`).
- Keep forcing legible in telemetry (`telemetry.rs` KPI: temp index, soil moisture, event forecast accuracy) — the Data Scientist tester can then measure whether adaptation mechanics actually change outcomes.

## Rating

High leverage: one system that pressures every economy the game has.
