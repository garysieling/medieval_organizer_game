# Academic → Game Mechanic Translations

**Agent:** Literature Survey Agent
**Last Updated:** 2026-10-31
**Version:** 0.2.0

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

### Pattern: Historical Organizing Campaigns → Campaign Structure Mechanics
**Academic Source:** Alinsky, Ganz, McAlevey, Han (organizing theory); Chenoweth & Stephan NAVCO (quant); movement case histories (Birmingham, Delano, apartheid, Stonewall, suffrage, ordination, IAF, labor, abolition, climate)
**Game Mechanic:**
- Campaign structure: target org, winnable demand, escalation ladder (petition → structure test → strike/boycott → confrontation), settlement step
- Pillars of support: power-holder loyalty tracked per pillar (economy, guard, clergy, media, bureaucracy), pulled by targeted actions rather than a single "defeat the leader" bar
- Political jiu-jitsu / repression backfire: disciplined action met with visible repression converts fence-sitters and can trigger pillar defection; undisciplined action flips the roll in the power-holder's favor
- Institutional anchoring: orgs tied to a durable institution (church, guild, mutual aid society) decay slowly and survive a leader's death; unanchored orgs decay fast after a win
- Leadership pipeline vs. charismatic bottleneck: leader capacity caps, with trained successors multiplying org capacity and insuring against decapitation
- Ask specificity: specific demands win more often but cap low; diffuse demands fail more but can still shift the Overton frame
- Coalition breadth: alliances raise resilience and participation but add coordination friction and ideological-drift risk
**Implementation:** Proposed, not yet implemented - see `wiki/literature-reviews/organizing-historical-campaigns.md` (Game Mechanic Proposals 1-7); builds on the existing `Campaign`/`hold_campaign_meeting` model in `game2-core/src/game/campaign.rs`
**Simplification:** Pillar loyalty and discipline as a handful of tracked scalars rather than the full historical case-by-case dynamics
**Educational Value:** Shows organizing as a structural contest over named sources of a power-holder's support, not a morale bar
**Fun Factor:** Escalation ladder and pillar-pulling give players real strategic choices with historically-grounded tradeoffs
**Tags:** organizing, campaign, pillars-of-support, repression-backfire, leadership, coalition

---

### Pattern: Counter-Organizing History → Repression & Counter-Intel Mechanics
**Academic Source:** Church Committee (COINTELPRO primary sources); corporate counter-organizing playbook (1870s-present); documented agent-provocateur, false-flag, and soft-repression cases
**Game Mechanic:**
- Repression tech tree (power-holder side): surveillance → infiltration → legal harassment → provocation → violent repression, each with escalating cost and exposure risk
- Trust/paranoia stat: infiltration and snitch-jacketing raise paranoia, which degrades org decision speed and recruitment and raises purge risk; countered by `verification_norms` and `transparency`
- Controlled-opposition events: a captive seat (works council, advisory board) that may be binding or merely performative, discovered through a detection mini-game
- Provocateur events: an infiltrator proposes a violent action at the worst moment; countered by trained marshals, verify-then-act process, or eject-on-suspicion (which feeds paranoia)
- Manufactured crisis + refutation window: a real or staged incident lets a power-holder declare emergency measures; a timed documentation/media/elite-ally response can expose it as a scandal before it normalizes
- Mohawk Valley playbook as counter-AI: power-holder factions run the documented 9-step de-legitimize → vigilante → "back to work" sequence as a scripted strategy for players to learn to interrupt early
- Counter-intel as player capability: documentation, relational vetting, and surveillance-exploitation as unlockable org abilities, with a Media-PA-break-in-style high-risk action able to end a repression program
**Implementation:** Proposed, not yet implemented - see `wiki/literature-reviews/organizing-countertactics.md` (Game Mechanic Proposals 1-7)
**Simplification:** Paranoia and detection reduced to a few tracked stats and a binary binding/performative check, rather than full case-specific infiltration dynamics
**Educational Value:** Teaches the real historical mechanism of repression (COINTELPRO, company unions, Mohawk Valley Formula) as a system with legible counters, not just villainy
**Fun Factor:** Detection mini-games and a timed refutation window create tense, information-asymmetric play
**Tags:** organizing, repression, paranoia, counter-intel, controlled-opposition, provocateur

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
