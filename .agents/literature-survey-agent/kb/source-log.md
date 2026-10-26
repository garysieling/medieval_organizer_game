# Source Log

**Agent:** Literature Survey Agent
**Last Updated:** 2026-10-25
**Version:** 0.1.0

## Summary
Tracks all sources found during literature surveys with credibility ratings.

## Source Rating Scale
- **A+**: Peer-reviewed, widely cited, foundational work
- **A**: Peer-reviewed, well-cited
- **B**: Academic but less cited, or well-regarded industry source
- **C**: Industry source or blog with good reputation
- **D**: Unverified or low-quality source

## Sources

### Psychology & Sociology

#### Bowlby - Attachment Theory (Foundational)
**Type:** Academic Book/Papers
**Domain:** Psychology
**Rating:** A+
**Year:** 1969-1980s
**Relevance:** Character relationship modeling, attachment styles
**Used In:** psychology.rs (AttachmentStyle)
**Notes:** Foundational attachment theory work. Widely cited, empirically validated.

---

#### Ainsworth - Attachment Styles
**Type:** Academic Research
**Domain:** Psychology
**Rating:** A+
**Year:** 1970s
**Relevance:** Secure, anxious, avoidant, disorganized attachment patterns
**Used In:** psychology.rs (AttachmentStyle enum)
**Notes:** Empirically validated attachment style taxonomy. Strange Situation paradigm.

---

### Game Design

#### Sweetser - "An Emergent Approach to Game Design" (2006)
**Type:** PhD Thesis
**Domain:** Game Design, Emergent Systems
**Rating:** A
**Year:** 2006
**Relevance:** Cellular automata for game world simulation
**Used In:** emergence.rs (CA-based systems)
**Notes:** EmerGEnT framework. Demonstrates emergence from simple local rules.

---

#### Dormans - "Simulating Mechanics to Study Emergence in Games" (2011)
**Type:** Academic Paper
**Domain:** Game Design, Economics
**Rating:** A
**Year:** 2011
**Relevance:** Machinations framework for modeling game economies
**Used In:** economics.rs (resource flow concepts)
**Notes:** Influential work on formal game economy modeling. Machinations tool widely used.

---

### Sociology & Politics

#### Community Organizing Literature (Various)
**Type:** Academic + Practitioner Sources
**Domain:** Sociology, Political Organizing
**Rating:** B (varied sources)
**Year:** Various
**Relevance:** 1-1 conversations, house meetings, power building tactics
**Used In:** community.rs (OneOnOne, HouseMeeting)
**Notes:** Mix of academic sociology and practitioner guides. Need to compile specific sources.
**TODO:** Identify specific foundational sources (Saul Alinsky, etc.)

---

#### Sociology of Religion Literature (Various)
**Type:** Academic
**Domain:** Sociology, Religion Studies
**Rating:** B (varied sources)
**Year:** Various
**Relevance:** Religious gatherings, social cohesion, radicalization
**Used In:** religion.rs, fundamentalism.rs
**Notes:** General sociology of religion concepts. Need specific source compilation.
**TODO:** Identify key papers on fundamentalism emergence, congregational effects

---

#### Anti-Colonialist and Liberation Theory (Various)
**Type:** Academic + Political Theory
**Domain:** Political Science, History
**Rating:** B (varied sources)
**Year:** Various
**Relevance:** Perspective shift, power asymmetry, liberation mechanics
**Used In:** player.rs (PlayerRole progression)
**Notes:** Broad literature base. Need specific source compilation.
**TODO:** Identify key texts (Fanon, Freire, etc.)

---

### Systems Biology & Pharmacology

#### QSP/ADME Literature (Various)
**Type:** Academic (Pharmacology)
**Domain:** Systems Biology, Pharmacology
**Rating:** A (domain-specific)
**Year:** Various
**Relevance:** Compartment-based illness and treatment modeling
**Used In:** illness.rs (IllnessStage, Treatment)
**Notes:** Quantitative Systems Pharmacology is well-established field. ADME (Absorption, Distribution, Metabolism, Excretion) is standard framework.
**TODO:** Identify specific foundational QSP papers for citation

---

### History & Economics

#### Medieval Economics Literature (Various)
**Type:** Historical Research
**Domain:** Economic History
**Rating:** B (varied sources)
**Year:** Various
**Relevance:** Medieval markets, trade, guilds, pricing
**Used In:** economics.rs (medieval market simulation)
**Notes:** Historical economics research. Need specific source compilation.
**TODO:** Identify key works on medieval trade, guild systems, market structures

---

## Source Gaps to Fill

### High Priority
1. **Community Organizing** - Need specific foundational texts (Alinsky, etc.)
2. **Sociology of Religion** - Need key papers on fundamentalism emergence
3. **Medieval Economics** - Need historical sources on market structures
4. **QSP** - Need foundational QSP papers for proper citation

### Medium Priority
1. **Liberation Theory** - Compile key anti-colonialist texts (Fanon, Freire)
2. **Game Design** - Additional sources on emergent gameplay
3. **Historical Medicine** - Herbology, apothecary practices

### Low Priority
1. **Medieval Architecture** - Regional building styles
2. **Textile History** - Fabric production tech tree
3. **Tea History** - Cultural and trade significance

---
