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

### Pattern: Movement Sociology → Internal Org Structure
**Academic Source:** Freeman "Tyranny of Structurelessness" (1972); Snow et al. frame alignment (ASR 1986); Polletta participatory democracy (2002); Highlander/CEP pedagogy records
**Game Mechanic:** Per-member `alignment` gate on participation; delegated seats with a shadow-elite failure mode; `DecisionRule` (majority vs consensus) trade-off dial; train-the-trainer skill replication
**Implementation:** proposed for game2-core/src/game/campaign.rs (C1-C3 of wiki/campaign-plan.md)
**Simplification:** 3-level alignment enum instead of continuous frame congruence; two decision rules + a size threshold for splinter risk; teaching gated on trainer skill bar
**Educational Value:** Why groups need structure to act; why phrasing an issue matters as much as picking it; how organizing skill multiplies
**Fun Factor:** Grumbling dissenters, shadow-elite takeovers, hidden schools — inside drama to match the outside fights
**Tags:** organizing, internal-structure, frame-alignment, consensus, training, delegation

---

### Pattern: Prison History → Dungeon-as-Venue Mechanics
**Academic Source:** Jasper & Poulsen moral shocks (Social Problems 1995); suffragette prison record (UK Parliament, primary); Attica manifesto + historiography; Amnesty adoption-group model
**Game Mechanic:** `Imprisoned` as an active state (demands/strike/petition); moral-shock stranger recruitment (second channel beside network ties); jailer's-dilemma response table; release-campaign pressure accumulator with elite-prestige multiplier
**Implementation:** proposed for game2-core/src/game/dungeon.rs + reactions.rs (C5-C6 of wiki/campaign-plan.md)
**Simplification:** 4-option jailer response table instead of real policy space; pressure = actions × time + prestige sum; shock broadcast = radius + tag-intersection roll
**Educational Value:** Why arresting organizers backfires; how prisoners keep organizing; why persistence beats spectacle for release
**Fun Factor:** The jailer's every option loses when the movement is ready — players learn to plan for the arrest, not avoid it
**Tags:** organizing, prison, dungeon, moral-shocks, release-campaign, bystander-recruitment

---

### Pattern: Farr's Nosology → Zymotic Disease Roster
**Academic Source:** William Farr, Statistical Nosology (Registrar-General 4th Annual Report, 1842); Hardy, *The Epidemic Streets* (OUP 1993); Mooney (SHM 2007)
**Game Mechanic:** `DiseaseKind`/`Zymotic` enum + static table replacing free-string disease names: each kind carries route (contact | waterborne | vector | occupational | foodborne), contagiousness, virulence, lethality, immunity_conferred, incubation, season
**Implementation:** game2-core/src/sim/illness.rs (Disease.kind); spawn sites in session.rs look up kinds instead of hand-writing name+contagious+severity
**Simplification:** ~8 kinds to start (la grippe, scarlatina, the cholera, typhoid, putrid fever, consumption, ague, the gripes/ergotism) instead of the full roster; one route field instead of mixing
**Educational Value:** Cholera ≠ flu; the water/well was the enemy, not the neighbour
**Fun Factor:** Outbreaks have identity and counterplay; "the fever year" becomes "the scarlatina year"
**Tags:** disease, epidemiology, nosology, farr, zymotic, roster

---

### Pattern: Victorian Incubations → Stage::Latent Timer
**Academic Source:** Period incubation data (cholera 12h–5d, measles ~12d, scarlet fever 2–5d, typhoid 1–3wk) via Hardy and period clinical records
**Game Mechanic:** New infections spend `incubation` ticks in the existing-but-unused `Stage::Latent`: no damage, no (or weak) shedding; severity blooms after
**Implementation:** `Disease.incubation_left` in illness.rs; spread_contagion gates on stage
**Simplification:** One tick ≈ one day; per-kind constant, no variance
**Educational Value:** Why quarantine windows existed; the healthy-looking traveller is the scary one
**Fun Factor:** "Who else caught it?" dread; isolation decisions under uncertainty
**Tags:** disease, incubation, latent-stage, quarantine

---

### Pattern: Snow's Broad Street → Effluent/Water Transmission Route
**Academic Source:** Snow (1849/1854); Southwark & Vauxhall vs Lambeth natural experiment; Chadwick sewer-flushing backfire
**Game Mechanic:** Waterborne kinds deposit `effluent` on home/pit tiles; villagers drawing water/food from contaminated tiles gain exposure independently of proximity; cleaning or fixing the pit breaks the chain
**Implementation:** reuse `effluent: HashMap<Tile>` + `pit_water` already in session.rs (~line 2200); second channel beside spread_contagion
**Simplification:** Tile-level contamination, no water-flow simulation; effluent decays on a timer
**Educational Value:** THE Victorian insight — the pump, not the person; why sewers beat quarantine
**Fun Factor:** Find-the-pump detective moment; infrastructure as medicine
**Tags:** disease, waterborne, cholera, snow, sanitation, effluent

---

### Pattern: The Fever Calendar → Seasonal Outbreaks
**Academic Source:** Summer diarrhoea/cholera heat link (1890s peaks, 1911 epidemic); typhus/influenza winter link; ague autumn relapse; rickets late-spring; ergot damp-harvest (game already gates `tick % 365 == 210`)
**Game Mechanic:** Per-kind `season` window on the year cycle; outbreak spawner rolls only inside it; heatwave modifiers boost diarrhoeal kinds
**Implementation:** generalize FLU_DAY + ergot gates into a `seasonal_outbreaks` table
**Simplification:** Single annual window per kind; fixed probability per year
**Educational Value:** Victorians feared August for the belly and January for the chest — opposite seasons for opposite routes
**Fun Factor:** Predictable dread; stockpile herbs before the fever season
**Tags:** disease, seasonality, calendar, outbreaks

---

### Pattern: Typhus Respects the Rent → Crowding/Class Exposure
**Academic Source:** Liverpool typhus data (93% of deaths among weekly-wage earners); Hardy's crowding-susceptible childhood diseases; rickets as fatality amplifier
**Game Mechanic:** Exposure multiplier for co-housed villagers; Hunger lowers effective immunity (malnutrition→disease loop)
**Implementation:** spread_contagion already has Pos; add household co-location bonus + Hunger-based immunity debuff
**Simplification:** One crowding multiplier + one hunger threshold
**Educational Value:** Poverty IS a transmission route — the same germ killed the poor more
**Fun Factor:** Housing and wages become health policy; epidemics punish inequality the player built
**Tags:** disease, class, crowding, hunger, typhus, inequality

---

### Pattern: Phossy Jaw → Vocation Trade-Exposure
**Academic Source:** Match-factory phosphorus necrosis (USURJ); Pott 1775 sweep's cancer (first occupational cancer); Ramazzini/Agricola dust diseases; ILO C18 list
**Game Mechanic:** Long service in a `Vocation` accrues `trade_exposure`; past a threshold the trade's occupational disease sets in — chronic, non-contagious, skill-reducing
**Implementation:** `Craft.exposure` accumulation in vocation tick; threshold → Disease { kind: occupational }
**Simplification:** 3 trades to start (matches→phossy jaw, mining→the dust, sweeping→sweeps' rot); no cure, only prevention/switching trades
**Educational Value:** Industrial disease hid for decades — "it was just toothache"; why the 1888 matchgirls struck
**Fun Factor:** Guild reform vs output tension; the strike writes itself
**Tags:** disease, occupational, vocation, phossy-jaw, labor, matchgirls

---

### Pattern: Bread-Blame → Epidemic Scapegoating
**Academic Source:** Ergot/St Anthony's fire bread-blame (implemented); cholera blamed on migrants/Irish/railways; "fever nests" rhetoric; Snow legend > data
**Game Mechanic:** During outbreaks, villagers generate Grievance::Blamed toward plausible-but-wrong targets (baker, traveller, herb-wife, newcomers); identifying the true source is a player knowledge unlock
**Implementation:** extend ergot()'s blame pattern to other kinds; Problem::Blamed exists
**Simplification:** One wrong-target pick by proximity/novelty; no rumor-propagation model
**Educational Value:** Public health history is a history of blaming the wrong thing — and the wrong people
**Fun Factor:** Witch-hunt risk mid-epidemic; the player's info edge is actionable
**Tags:** disease, blame, grievance, scapegoat, miasma-minded

---
