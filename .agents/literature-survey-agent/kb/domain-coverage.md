# Domain Coverage

**Agent:** Literature Survey Agent
**Last Updated:** 2026-10-25
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
**Coverage:** Medium
**Surveys:** Sprint 3 (Community Organizing)
**Mechanics Implemented:** 1-1 conversations, house meetings (community.rs)
**Sources:** Community organizing literature (needs specific citations)
**Next Steps:** Coalition building, direct action tactics, movement scaling

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
**Next Steps:** Syncretism, heresy, religious reform movements

---

#### Political Theory & Liberation
**Coverage:** Medium-Low
**Surveys:** Sprint 4 (Anti-Colonialist/Liberation)
**Mechanics Implemented:** Player role progression (Outsider → Leader)
**Sources:** Liberation theory (needs specific citations)
**Next Steps:** Specific organizing tactics, oppression mechanics, revolution dynamics

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

#### Anthropology
**Coverage:** Low
**Surveys:** None dedicated
**Mechanics Implemented:** None directly
**Gaps:**
- Cultural evolution
- Kinship systems
- Ritual and ceremony
- Material culture
**Priority:** Medium
**Next Survey:** Cultural systems and rituals

---

#### Ecology & Climate
**Coverage:** Low
**Surveys:** Some climate basics
**Mechanics Implemented:** Basic climate (climate.rs - stub)
**Gaps:**
- Ecosystem dynamics
- Climate change modeling
- Resource depletion
- Biodiversity
**Priority:** High
**Next Survey:** Ecology and climate science for game mechanics

---

### Uncovered Domains ✗
Domains with no research yet but relevant to game vision.

#### Military History & Strategy
**Coverage:** None
**Gaps:** Battle tactics, fortification, logistics, military organization
**Priority:** Low-Medium (simple battle system planned)
**Next Survey:** When implementing detailed military systems

---

#### Philosophy & Ethics
**Coverage:** None
**Gaps:** Ethical frameworks, trolley problems, moral philosophy
**Priority:** Medium (trolley problem mechanics planned)
**Next Survey:** Ethical dilemmas as game mechanics

---

#### Linguistics & Communication
**Coverage:** None
**Gaps:** Language evolution, literacy, communication systems
**Priority:** Low
**Next Survey:** When implementing printmaking/literacy tech tree

---

#### Art History & Aesthetics
**Coverage:** Low (medieval theming only)
**Gaps:** Art movements, aesthetic evolution, cultural taste
**Priority:** Low
**Next Survey:** When expanding art direction

---

#### Education & Pedagogy
**Coverage:** None
**Gaps:** Learning systems, skill acquisition, knowledge transfer
**Priority:** Medium
**Next Survey:** When implementing education/skill systems

---

#### Urban Planning
**Coverage:** None
**Gaps:** City layout, infrastructure, public spaces, zoning
**Priority:** Medium
**Next Survey:** When scaling from village to city

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

**Cycle 2 (In Progress):**
1. **Agriculture & Food Systems** ✓ (COMPLETE - core medieval gameplay)
2. **Ecology & Climate** (HIGH PRIORITY - environmental simulation - NEXT)
3. **Urban Planning** (MEDIUM - village to city scaling)
4. **Philosophy & Ethics** (MEDIUM - trolley problem mechanics)
5. **Anthropology** (MEDIUM - cultural systems)
6. **Historical Events** (MEDIUM - dynamic world events)

**Cycle 3 (Future):**
1. Military History & Strategy (when needed)
2. Linguistics & Communication (literacy/printmaking systems)
3. Art History & Aesthetics (expanded art direction)
4. Education & Pedagogy (skill/education systems)
5. Magic Systems from Fiction (novel powers)
6. Theory of Mind (advanced psychology)

---

## Hot-Pursuit Topics
Topics that emerge from other work and need immediate research.

**Current Hot-Pursuit:**
- None active

**Completed Hot-Pursuit:**
- None yet

---

## Research Quality Metrics

**Total Surveys Completed:** 8
**Domains Covered (High/Medium):** 8
**Domains Partially Covered:** 1 (History - reduced from 2)
**Domains Uncovered:** 6 (reduced from 7)
**Mechanics Implemented from Research:** ~12 (5 new mechanics proposed for agriculture)
**Sources Requiring Better Citation:** ~5

**Coverage Score:** 50% (8 well-covered / 16 total relevant domains)
**Target for Next Sprint:** 56% (add 1 new domain from Cycle 2)

---
