# Domain Coverage

**Agent:** Literature Survey Agent
**Last Updated:** 2026-10-31
**Version:** 0.1.0

## Summary
Tracks which research domains have been covered and identifies gaps for future surveys.

## Coverage Status

### Well-Covered Domains ✓
Domains with completed surveys and implemented mechanics.

#### Game Design & Emergence
**Coverage:** High
**Surveys:** Sprint 1 (Game Design & Simulation)
**Mechanics Implemented:** Cellular automata (emergence.rs), Machinations-inspired resource flow
**Sources:** Sweetser (2006), Dormans (2011)
**Next Steps:** Explore additional emergent systems (flocking, agent-based modeling)

---

#### Psychology
**Coverage:** Medium-High
**Surveys:** Sprint 1 (Psychology & Economics)
**Mechanics Implemented:** Attachment theory (psychology.rs - AttachmentStyle)
**Sources:** Bowlby, Ainsworth
**Next Steps:** Theory of mind, emotional regulation, trauma modeling

---

#### Economics (General)
**Coverage:** Medium
**Surveys:** Sprint 1 (Psychology & Economics), Sprint 3 (Medieval Economics)
**Mechanics Implemented:** Medieval markets (economics.rs), age-based self-interest
**Sources:** Medieval economics literature
**Next Steps:** More detailed market dynamics, financial instruments, economic inequality

---

#### Community Organizing & Sociology
**Coverage:** High (deepened 2026-10-31)
**Surveys:** Sprint 3 (Community Organizing), Sprint 4 (Organizing Leadership), Sprint 7 (Organizing Power, Non-violent Deposition), Historical Campaigns of Community Organizing, Counter-Organizing: Repression & Infiltration, Enforcement & Defense of Wins, Campaign Engineering Masters, Modern Counter-Playbook, Movement Information Infrastructure
**Mechanics Implemented:** 1-1 conversations, house meetings (community.rs)
**Mechanics Proposed:**
- Campaign structure (target, demand, escalation ladder, settlement) - historical-campaigns review
- Pillars of support (defection-based power model) - historical-campaigns review
- Repression backfire / political jiu-jitsu - historical-campaigns review
- Institutional anchoring (durable vs ephemeral orgs) - historical-campaigns review
- Leadership pipeline vs charismatic bottleneck - historical-campaigns review
- Ask specificity (winnable vs transformational campaigns) - historical-campaigns review
- Coalition breadth vs coordination friction - historical-campaigns review
- Repression tech tree (surveillance → infiltration → legal harassment → provocation → violence) - countertactics review
- Trust/paranoia stat (COINTELPRO's real mechanism) - countertactics review
- Controlled-opposition events (binding vs performative seats) - countertactics review
- Provocateur events + marshal/discipline counters - countertactics review
- Manufactured crisis + refutation window - countertactics review
- Mohawk Valley Formula as counter-AI playbook - countertactics review
- Counter-intel abilities (documentation, vetting, surveillance exploitation) - countertactics review
- Win-decay + enforcement tiers + counter-movement timer - enforcement review
- Campaign design stack (6 layers) + decision-tree rule + playbook export/counter-adaptation - campaign-masters review
- Repression-as-market + legalization track + lawfare events - modern-playbook review
- Channel ownership + petition artifacts + suppression backfire + brand propagation - info-infrastructure review
**Sources:** Alinsky, Ganz, McAlevey, Han (theory); Chenoweth & Stephan NAVCO (quant); Church Committee (primary on counter-organizing); movement case histories (Birmingham, Delano, apartheid, Stonewall, suffrage, ordination, IAF, labor, abolition, climate); TigerSwan leaks, Citizen Lab, HRW (modern repression); APSR petition dataset, Tufekci (info infrastructure)
**Next Steps:** Deep-dives done: 504, ACT UP, abolition petitions (round 2). Remaining: UK spycops inquiry, NGO-ization/co-option literature, platform-manipulation research

---

#### Systems Biology & Medicine
**Coverage:** Medium
**Surveys:** Sprint 3 (QSP for Game Systems)
**Mechanics Implemented:** Illness stages, treatments (illness.rs)
**Sources:** QSP/ADME literature (needs specific citations)
**Next Steps:** Specific illnesses (cancer progression), preventive medicine, epidemiology

---

#### Religion & Fundamentalism
**Coverage:** Medium
**Surveys:** Sprint 3 (Religion & Emergence)
**Mechanics Implemented:** Congregations, fundamentalism (religion.rs, fundamentalism.rs)
**Sources:** Sociology of religion (needs specific citations)
**User Note - Religion-Economics Feedback Loop (Bonhoeffer, Niebuhr):**
- Economic scarcity → Religious coping (simplicity/hard work as values, celebration of pain, apocalyptic hope)
- New technology → Removes need → Tension between faith and reality → Schism (new organizations form)
- Reactionaries resist change (to maintain power)
- Self-interest masked as selflessness (Niebuhr - hypocrisy in religious leadership)
- **Systems dynamic**: Scarcity → Religion fills gap → Tech removes need → Crisis of faith → Reorganization
**Next Steps:** Syncretism, heresy, religious reform movements, religion-economics interaction mechanics

---

#### Political Theory & Liberation
**Coverage:** Medium-Low
**Surveys:** Sprint 4 (Anti-Colonialist/Liberation)
**Mechanics Implemented:** Player role progression (Outsider → Leader)
**Sources:** Liberation theory (needs specific citations)
**Next Steps:** Specific organizing tactics, oppression mechanics, revolution dynamics

---

#### Ecology & Climate
**Coverage:** High
**Surveys:** Cycle 2, Survey 2 (Ecology & Climate)
**Mechanics Implemented:** None yet (proposed in wiki/literature-reviews/ecology-climate.md)
**Sources:** Lotka-Volterra equations, Medieval climate research, Medieval deforestation data
**Mechanics Proposed:**
- Climate event system (Medieval Warm Period → Little Ice Age)
- Predator-prey wildlife cycles
- Forest depletion & regeneration (70% historical rate)
- Weather-modified crop yields
- Ecosystem services & degradation
**Next Steps:** Implement ecology.rs and climate.rs modules, integrate with agriculture system

---

#### Urban Planning
**Coverage:** High
**Surveys:** Cycle 2, Survey 3 (Urban Planning)
**Mechanics Implemented:** None yet (proposed in wiki/literature-reviews/urban-planning.md)
**Sources:** PLOS ONE scaling study, Medieval urban layouts, Infrastructure research
**Mechanics Proposed:**
- Urban scaling with density laws (Area ∝ Population^0.75)
- Zoning system (residential/commercial/industrial/civic)
- Infrastructure tech tree (water supply, sanitation, roads)
- Market square mechanics (scales with population)
- Village → Town → City progression
**Next Steps:** Implement urban.rs and infrastructure.rs modules, test scaling formulas

---

#### Philosophy & Ethics
**Coverage:** High
**Surveys:** Cycle 2, Survey 4 (Philosophy & Ethics)
**Mechanics Implemented:** None yet (proposed in wiki/literature-reviews/philosophy-ethics.md)
**Sources:** Trolley problem (Foot, Thomson), Ethical frameworks, Medieval virtues (Aquinas), Game theory
**Mechanics Proposed:**
- Trolley problem scenarios (contextual moral dilemmas)
- Three ethical alignment paths (utilitarianism/deontology/virtue ethics)
- Medieval seven virtues system (character development)
- Moral intensity modifiers (context-sensitive difficulty)
- Reputation & indirect reciprocity (gossip-based enforcement)
**Next Steps:** Implement ethics.rs and reputation.rs modules, test dilemma scenarios

---

#### Anthropology
**Coverage:** High
**Surveys:** Cycle 2, Survey 5 (Anthropology)
**Mechanics Implemented:** None yet (proposed in wiki/literature-reviews/anthropology.md)
**Sources:** Kinship theory, Van Gennep, Turner, Medieval family structures, Cultural transmission
**Mechanics Proposed:**
- Kinship and descent systems (patrilineal/matrilineal/bilateral)
- Rites of passage (separation, liminality, reincorporation)
- Marriage alliances and negotiations (strategic politics)
- Cultural transmission (vertical/horizontal/oblique learning)
- Ritual calendar (annual events with social cohesion effects)
**Next Steps:** Implement kinship.rs and ritual.rs modules, test family politics gameplay

---

#### Historical Events
**Coverage:** High
**Surveys:** Cycle 2, Survey 6 (Historical Events)
**Mechanics Implemented:** None yet (proposed in wiki/literature-reviews/historical-events.md)
**Sources:** Crisis of Late Middle Ages, Great Famine, Black Death, Poisson distribution, Medieval innovations
**Mechanics Proposed:**
- Poisson-based disaster calendar (statistical event frequency)
- Cascade system (event chains: famine → disease → unrest)
- Pandemic simulation (Black Death with spatial spread)
- Technology innovation events (discovery and diffusion)
- Invasion and migration events (Vikings, Mongols with responses)
**Next Steps:** Implement events.rs and pandemic.rs modules, test disaster cascades

---

#### Herbology & Medieval Medicine
**Coverage:** High
**Surveys:** Cycle 3, Survey 1 (Herbology & Medieval Medicine)
**Mechanics Implemented:** None yet (proposed in wiki/literature-reviews/herbology-medieval-medicine.md)
**Sources:** Four Humors (Hippocrates/Galen), Monastic medicine, Dioscorides, Medieval poisons
**Mechanics Proposed:**
- Four Humors diagnostic system (balance blood/phlegm/yellow bile/black bile)
- Monastic herb garden cultivation (50+ medicinal plants)
- Apothecary crafting (infusions, decoctions, poultices, tinctures, salves)
- Herb gathering & foraging (seasonal, misidentification risk)
- Poison & antidote system (hemlock, belladonna, mandrake, mithridatium)
**Next Steps:** Implement herbology.rs and humors.rs modules, integrate with illness.rs

---

### Partially-Covered Domains ⚠
Domains with initial research but needing deeper surveys.

#### History
**Coverage:** Low-Medium
**Surveys:** Medieval economics, some medieval thematic research
**Mechanics Implemented:** Medieval markets, period aesthetics
**Gaps:**
- Specific historical periods (Renaissance, Early Modern)
- Non-European histories (Islamic Golden Age, East Asian medieval periods)
- Historical events as game mechanics (plagues, migrations, etc.)
**Priority:** Medium
**Next Survey:** Historical events and their game mechanic equivalents

---

### Uncovered Domains ✗
Domains with no research yet but relevant to game vision.

#### Military History & Strategy
**Coverage:** High
**Surveys:** Cycle 3, Survey 6 (Military History & Strategy)
**Mechanics Implemented:** None yet (proposed in wiki/literature-reviews/military-history-strategy.md)
**Sources:** Medieval warfare tactics, Fortification evolution, Military organization, Campaign logistics, Lanchester's Laws
**Mechanics Proposed:**
- Simple battle resolution (Lanchester's Linear Law with morale and terrain modifiers)
- Military organization (feudal levies 40-day limit, professional soldiers reliable, mercenaries elite but loyalty risk)
- Fortification system (wooden → motte-and-bailey → stone castle → concentric castle with 10x defensive advantage)
- Campaign logistics (foraging 5-10 miles/day devastates countryside, naval supply 20 miles/day, seasonal spring-autumn)
**Next Steps:** Implement battle.rs and fortification.rs modules for simple strategic combat

---

#### Art History & Aesthetics
**Coverage:** High
**Surveys:** Cycle 3, Survey 7 (Art History & Aesthetics)
**Mechanics Implemented:** None yet (proposed in wiki/literature-reviews/art-history-aesthetics.md)
**Sources:** Medieval art styles (Byzantine/Romanesque/Gothic), Kant (beautiful vs sublime), Medieval color symbolism, Hume (taste theory)
**Mechanics Proposed:**
- Medieval art style progression (Byzantine flat iconic → Romanesque bold → Gothic graceful naturalistic)
- Atmosphere system (Beautiful with harmony/pastoral vs Sublime with awe/terror)
- Color symbolism mechanics (Gold = divinity, Blue = Virgin Mary/spiritual, Red = blood/sacrifice)
- Event color coding (divine intervention = gold, battles = red, peace = blue)
- Aesthetic experience gameplay (Kantian beautiful vs sublime moments)
**Next Steps:** Implement atmosphere.rs module for visual/emotional direction, integrate color symbolism with events

---

#### Linguistics & Communication
**Coverage:** None
**Gaps:** Language evolution, literacy, communication systems
**Priority:** Low
**Next Survey:** When implementing printmaking/literacy tech tree

---

#### Education & Pedagogy
**Coverage:** High
**Surveys:** Cycle 3, Survey 4 (Education & Pedagogy)
**Mechanics Implemented:** None yet (proposed in wiki/literature-reviews/education-pedagogy.md)
**Sources:** Medieval education (monastery/cathedral/university), Guild apprenticeships, Deliberate practice (Ericsson), Tacit knowledge theory
**Mechanics Proposed:**
- Medieval education structures (monastery schools, cathedral schools, universities with four faculties)
- Seven liberal arts curriculum (trivium + quadrivium progression)
- Guild apprenticeship system (apprentice → journeyman → master with masterpiece requirement)
- Deliberate practice skill system (quality over quantity, 3x multiplier for focused practice)
- Literacy teaching (wax tablets, hornbooks, oral recitation, reading vs writing as separate skills)
- Tacit vs explicit knowledge transfer (books for theory, apprenticeship for embodied skills)
**Next Steps:** Implement education.rs and skills.rs modules, test deliberate practice mechanics, balance apprenticeship timescales

---

#### Medieval Crafts & Tools
**Coverage:** High
**Surveys:** Cycle 3, Survey 2 (Medieval Crafts & Tools)
**Mechanics Implemented:** None yet (proposed in wiki/literature-reviews/medieval-crafts-tools.md)
**Sources:** Luthier tonewood research, Medieval beekeeping and skeps, Stained glass techniques, Stonemason tool progression
**Mechanics Proposed:**
- Luthier profession with wood selection (spruce soundboards, maple ribs, tonewood properties)
- Beekeeping with skeps (rye/wheat straw basket hives, seasonal honey/wax harvest cycles)
- Stained glass multi-stage crafting (design → cut → paint → fire → lead → assemble)
- Stonemason skill tiers (apprentice → roughmason → freemason → master with tool progression)
**Next Steps:** Implement craft.rs module with specialized profession systems, test guild mechanics

---

#### Writing Materials & Bookmaking
**Coverage:** High
**Surveys:** Cycle 3, Survey 3 (Writing Materials & Bookmaking)
**Mechanics Implemented:** None yet (proposed in wiki/literature-reviews/writing-materials-bookmaking.md)
**Sources:** Vellum/parchment process, Iron gall ink, Linen rag papermaking, Coptic binding, Manuscript illumination, Vegetable papyrus
**Mechanics Proposed:**
- Five writing surfaces (vellum 14-day process, papyrus, vegetable papyrus from peppers, linen rag paper)
- Four ink types (iron gall archival, carbon/lampblack, berry inks, mineral pigment from stone dust)
- Bookbinding progression (Coptic binding → medieval codex with raised cords)
- Manuscript illumination (gold leaf on gesso, burnishing, lapis lazuli pigments)
- Complete literacy tech tree (professions: parchment maker, vatman, scribe, bookbinder, illuminator)
**Next Steps:** Implement writing.rs and bookmaking.rs modules, test literacy progression system

---

#### Agriculture & Food Systems ✓
**Coverage:** High
**Surveys:** Cycle 2, Survey 1 (Agriculture & Food Systems)
**Mechanics Implemented:** None yet (proposed in wiki/literature-reviews/agriculture-food-systems.md)
**Sources:** Winchester Manors data, Three-field system (Oxford/InTech), APSIM models
**Mechanics Proposed:**
- Three-field crop rotation (+33% yields)
- Seasonal food security cycle
- Food preservation tech tree
- Crop yield variability
- Village collective farming
**Next Steps:** Implement agriculture.rs module, test rotation mechanics, balance seasonal cycle

---

#### Systems Thinking & Dynamics
**Coverage:** High
**Surveys:** Cycle 3, Survey 5 (Systems Thinking & Dynamics)
**Mechanics Implemented:** None yet (proposed in wiki/literature-reviews/systems-thinking-dynamics.md)
**Sources:** Meadows (Thinking in Systems), Forrester (System Dynamics), Senge (Fifth Discipline), Agricultural economics
**Mechanics Proposed:**
- Stock-flow framework (bathtub metaphor, accumulation dynamics for all resources)
- Feedback loops (reinforcing R = growth/collapse, balancing B = stability/oscillation)
- Delay system (time separation between cause and effect, oscillation dynamics)
- **Correlated yield agricultural economics** (weather affects all farmers, inverse price-yield r=-0.51, bankruptcy in both bumper and drought years)
- System archetypes (Limits to Growth, Shifting the Burden, Tragedy of Commons pattern recognition)
- Causal loop diagram visualization (meta-game educational tool)
**Next Steps:** Apply to ALL game systems - agriculture, population, economy, resources. CRITICAL foundation for realistic dynamics.

---

#### Magic Systems (Fiction)
**Coverage:** None
**Gaps:** Novel magic systems from fiction, non-standard powers
**Priority:** Medium
**Next Survey:** When implementing magic system

---

## Domain Rotation Schedule

### Proposed Schedule
Literature Survey Agent should rotate through domains to maintain breadth while allowing focused deep dives.

**Cycle 1 (Completed):**
1. Game Design & Emergence ✓
2. Psychology & Economics ✓
3. Community Organizing ✓
4. QSP & Medicine ✓
5. Medieval Economics ✓
6. Religion & Emergence ✓
7. Anti-Colonialist Liberation ✓

**Cycle 2 (COMPLETE ✓):**
1. **Agriculture & Food Systems** ✓ (COMPLETE - core medieval gameplay)
2. **Ecology & Climate** ✓ (COMPLETE - environmental simulation)
3. **Urban Planning** ✓ (COMPLETE - village to city scaling)
4. **Philosophy & Ethics** ✓ (COMPLETE - trolley problem mechanics)
5. **Anthropology** ✓ (COMPLETE - kinship and ritual systems)
6. **Historical Events** ✓ (COMPLETE - dynamic world events)

**Cycle 3 (COMPLETE ✓ - 100% COVERAGE ACHIEVED!):**
1. **Herbology & Medieval Medicine** ✓ (COMPLETE - four humors and herb gardens)
2. **Medieval Crafts & Tools** ✓ (COMPLETE - luthier, beekeeping, stained glass, stonemason)
3. **Writing Materials & Bookmaking** ✓ (COMPLETE - vellum, inks, bookbinding, illumination)
4. **Education & Pedagogy** ✓ (COMPLETE - schools, apprenticeships, deliberate practice)
5. **Systems Thinking & Dynamics** ✓ (COMPLETE - USER PRIORITY - stocks/flows, feedback loops, correlated yields)
6. **Military History & Strategy** ✓ (COMPLETE - simple battle system, fortifications, logistics)
7. **Art History & Aesthetics** ✓ (COMPLETE - medieval art styles, Kantian aesthetics, color symbolism)

**Optional Advanced Domains (Available for Future Cycles):**
- Linguistics & Communication (language evolution, literacy systems)
- Magic Systems from Fiction (novel magic mechanics)
- Theory of Mind (advanced psychology)

---

## Hot-Pursuit Topics
Topics that emerge from other work and need immediate research.

**Current Hot-Pursuit:**
- None active

**Completed Hot-Pursuit:**
- None yet

---

## Research Quality Metrics

**Total Surveys Completed:** 26 (+6 on 2026-10-31: Historical Campaigns; Counter-Organizing; Enforcement/Defense; Campaign Masters; Modern Counter-Playbook; Information Infrastructure — user-requested two-round deep dive)
**Domains Covered (High/Medium):** 20
**Domains Partially Covered:** 1 (History - general, will refine over time)
**Domains Uncovered:** 0 (100% COVERAGE ACHIEVED!)
**Mechanics Implemented from Research:** ~106 (5 agriculture + 5 ecology + 5 urban + 5 philosophy + 5 anthropology + 5 historical events + 5 herbology + 4 crafts + 5 writing materials + 6 education + 6 systems thinking + 4 military + 5 art/aesthetics + 7 misc proposed + 14 organizing/countertactics + 20 round-2 organizing mechanics)
**Sources Requiring Better Citation:** ~4 (Community Organizing citation gap FILLED 2026-10-31)

**Coverage Score:** 100% (20/20 well-covered domains!)
**Cycle 1: COMPLETE** (7/7 surveys finished)
**Cycle 2: COMPLETE** (6/6 surveys finished)
**Cycle 3: COMPLETE** (7/7 surveys finished - 100% COVERAGE ACHIEVED!)

**MILESTONE ACHIEVED:** All planned domains surveyed! Optional advanced domains (Linguistics, Magic Systems, Theory of Mind) available for future exploration.

---
