# Academic → Game Mechanic Translations

**Agent:** Literature Survey Agent
**Last Updated:** 2026-10-25
**Version:** 0.1.0

## Summary
Maps academic concepts and research findings to concrete game mechanics.

## Translation Patterns

### Pattern: Psychological Model → Character Trait System
**Academic Source:** Attachment Theory (Bowlby, Ainsworth)
**Game Mechanic:** Character attachment style trait (Secure, Anxious, Avoidant, Disorganized)
**Implementation:** game2-core/src/sim/psychology.rs - `AttachmentStyle` enum
**Simplification:** Discrete categories instead of continuous dimensions
**Educational Value:** Players see how attachment affects relationships
**Fun Factor:** Creates varied character behaviors and relationship dynamics
**Tags:** psychology, traits, relationships

---

### Pattern: Systems Biology → Illness Simulation
**Academic Source:** ADME/QSP (Quantitative Systems Pharmacology)
**Game Mechanic:** Compartment-based illness progression with treatments
**Implementation:** game2-core/src/sim/illness.rs - `IllnessStage`, `Treatment`
**Simplification:** Simplified compartments, discrete stages vs continuous dynamics
**Educational Value:** Teaches how medicines work on body systems
**Fun Factor:** Depth without overwhelming complexity
**Tags:** qsp, illness, medicine, compartments

---

### Pattern: Community Organizing → Influence Building
**Academic Source:** Community organizing principles
**Game Mechanic:** 1-1 conversations and house meetings build power
**Implementation:** game2-core/src/sim/community.rs - `OneOnOne`, `HouseMeeting`
**Simplification:** Influence as emergent effect, not explicit stat
**Educational Value:** Shows how grassroots organizing works
**Fun Factor:** Player sees power build through relationships
**Tags:** organizing, influence, relationships

---

### Pattern: Cellular Automata → World Simulation
**Academic Source:** Sweetser's EmerGEnT system (2006)
**Game Mechanic:** Simple local rules create emergent world behavior
**Implementation:** game2-core/src/sim/emergence.rs - CA-based systems
**Simplification:** 2D grid, simple rules (fire spread, resource depletion)
**Educational Value:** Demonstrates emergence from simple rules
**Fun Factor:** Unpredictable world creates variety
**Tags:** cellular-automata, emergence, world-simulation

---

### Pattern: Resource Economics → Flow System
**Academic Source:** Machinations framework (Dormans, 2011)
**Game Mechanic:** Resource nodes, converters, pools with flow rates
**Implementation:** game2-core/src/sim/economics.rs - market and trade systems
**Simplification:** Discrete turns instead of continuous flow
**Educational Value:** Shows economic system dependencies
**Fun Factor:** Strategic resource management
**Tags:** economics, resources, machinations

---

### Pattern: Sociology of Religion → Belief Systems
**Academic Source:** Sociology of religion literature
**Game Mechanic:** Religious gatherings create social cohesion + radicalization risk
**Implementation:** game2-core/src/sim/religion.rs + fundamentalism.rs
**Simplification:** Congregation strength + fundamentalism as separate values
**Educational Value:** Shows how religion affects society (not just individual belief)
**Fun Factor:** Emergent religious movements, satire opportunity (Bioshock-style)
**Tags:** religion, sociology, emergence, radicalization

---

### Pattern: Historical Economics → Medieval Trade
**Academic Source:** Medieval market history
**Game Mechanic:** Supply/demand pricing, guild monopolies, trade routes
**Implementation:** game2-core/src/sim/economics.rs - medieval market simulation
**Simplification:** Simplified pricing formulas, discrete goods
**Educational Value:** Teaches medieval economic structures
**Fun Factor:** Dynamic economy creates strategic opportunities
**Tags:** medieval, economics, trade, guilds

---

### Pattern: Liberation Theory → Power Asymmetry
**Academic Source:** Anti-colonialist and liberation literature
**Game Mechanic:** Start as outsider with no authority, build power through organizing
**Implementation:** game2-core/src/game/player.rs - `PlayerRole::Outsider`
**Simplification:** Discrete authority levels, clear progression path
**Educational Value:** Teaches how power is built from the bottom up
**Fun Factor:** Satisfying progression from powerless to influential
**Tags:** liberation, power-building, progression

---

### Pattern: Hazardous Craft Zoning -> Outside-the-Walls Visitor
**Academic Source:** Pistoia stench-craft statute, English furnace-craft bylaws, Encyclopedie varnish siting advice, Cennini's stove
**Game Mechanic:** Periodic specialist (oil-boiler/ink-maker) camps a cauldron on the meadow outside the gate - the fire/smell hazard IS the reason for the location
**Implementation:** session.rs on the Circus spawn/advance/tick pattern
**Simplification:** One visitor per ~year, multi-day phases, small per-day fire chance
**Educational Value:** Why medieval towns zoned dangerous trades; the ink on the page starts as a fire on the meadow
**Fun Factor:** A scheduled spectacle with a real downside tail (the meadow burns)
**Tags:** ink, oil-boiling, fire, urban-zoning, periodic-events

---

### Pattern: Process-Stage Recipe -> Multi-Phase Event
**Academic Source:** Encyclopedie "Encre noir" recipe (boil -> ignite/smother -> bread + onions -> slate test -> grind black)
**Game Mechanic:** Event advances through historical phases, one per day, each with its own beat (smoke, fried crusts, the drip test)
**Implementation:** phase enum on the event struct
**Simplification:** 4-5 phases, no player micro-management
**Educational Value:** The recipe itself is the content - bread really did go into the oil
**Fun Factor:** Villagers gather; kids get fried crusts; the drip test is a small reveal
**Tags:** recipe-as-gameplay, ink, phased-events

---

### Pattern: Flammability Field -> Map Fire Spread
**Academic Source:** Medieval town conflagrations around furnace trades; Sweetser EmerGEnT CA
**Game Mechanic:** Burning tiles spread to flammable neighbours (grass, crops, trees, buildings), blocked by water/road, burn out to scorched ground
**Implementation:** new fire module over TileMap (emergence.rs proved the rules)
**Simplification:** Probabilistic spread, fixed burn duration, recovery over days
**Educational Value:** Why one cauldron could burn a medieval town
**Fun Factor:** Rare catastrophe everyone remembers
**Tags:** fire, cellular-automata, emergence

---

## Translation Heuristics

### Math → Mechanics Rule
When translating mathematical models:
1. Find the core mechanism (what changes what?)
2. Simplify to 2-5 key variables
3. Make changes discrete (turns) not continuous (differential equations)
4. Scale values to human-readable ranges (0-100, not 0.00234)
5. Make outcomes visible to player

### Educational but Fun Rule
1. Preserve the "why" (causal relationships)
2. Simplify the "how" (exact formulas)
3. Add feedback loops for interesting emergent behavior
4. Ensure player can experiment and learn

### Source Citation Rule
Every mechanic must trace to:
- Academic source (paper, book, research)
- OR historical source (documented practice)
- OR established game design pattern (with citation)

---
