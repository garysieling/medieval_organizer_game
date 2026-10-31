# Source Log

**Agent:** Literature Survey Agent
**Last Updated:** 2026-10-31
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

### Social Movements & Counter-Organizing (added 2026-10-31)

#### Chenoweth & Stephan - *Why Civil Resistance Works* + NAVCO dataset (2011)
**Type:** Academic Book + Dataset
**Domain:** Political Science, Social Movements
**Rating:** A+
**Year:** 2011
**Relevance:** Quantitative basis for nonviolent-vs-violent campaign outcomes (53% vs 26% success); pillars-of-support mechanism; 3.5% participation heuristic (descriptive, not prescriptive — see author's 2020 caveat paper)
**Used In:** organizing-historical-campaigns.md
**Notes:** Largest dataset of its kind (323 campaigns 1900-2006). Bahrain 2011 is documented exception to 3.5%.

#### Church Committee Reports, Books II-III (US Senate, 1976)
**Type:** Primary Government Investigation
**Domain:** Intelligence History, Repression
**Rating:** A+
**Year:** 1976
**Relevance:** Definitive documented record of COINTELPRO tactics (2,370 actions): snitch-jacketing, provocateurs, forged letters, fomented BPP-US violence, MLK suicide letter, Hampton raid floor plan, Jean Seberg op
**Used In:** organizing-countertactics.md
**Notes:** Exposed by the Citizens' Commission Media PA break-in (1971). The gold standard — everything else on FBI repression gets checked against it.

#### King Institute (Stanford) - Birmingham Campaign primary materials
**Type:** University archive, primary documents
**Domain:** Civil Rights History
**Rating:** A
**Year:** 1963 materials, curated
**Relevance:** Project C design (focus downtown business, engineered confrontation, exploited tapped phones for disinformation), Albany→Birmingham learning loop
**Used In:** organizing-historical-campaigns.md, organizing-countertactics.md

#### Global Nonviolent Action Database (Swarthmore)
**Type:** Peer-reviewed case database
**Domain:** Social Movements
**Rating:** A
**Year:** Ongoing
**Relevance:** Structured case records of campaigns incl. Delano grape strike (Schenley targeting logic)
**Used In:** organizing-historical-campaigns.md

#### Garrow - "FBI Political Harassment and FBI Historiography" (Public Historian, 1988)
**Type:** Peer-reviewed article
**Domain:** Intelligence History
**Rating:** A
**Year:** 1988
**Relevance:** Effects of informants/provocateurs; Albertson snitch-jacket detail, Seberg case
**Used In:** organizing-countertactics.md

#### Smith & Tennent - ERP movement 1913-1935 (Labor History, 2025); Kaufman - company unions (Industrial Relations, 2016)
**Type:** Peer-reviewed articles
**Domain:** Labor History
**Rating:** A / B+
**Year:** 2016, 2025
**Relevance:** Company unions/ERPs as controlled opposition; Wagner Act ban; legitimation strategy
**Used In:** organizing-countertactics.md
**Notes:** Kaufman argues ban was overbroad (revisionist) — useful for the performative-vs-binding seat mechanic

#### "Union Avoidance: Rationale, Strategies, and Practices" (Southeastern Oklahoma State U working paper)
**Type:** Working paper
**Domain:** Labor Relations
**Rating:** B+
**Year:** undated
**Relevance:** Documents Mohawk Valley Formula 9-step strikebreaking playbook, American Plan
**Used In:** organizing-countertactics.md
**Notes:** MVF also documented in NLRB v. Remington Rand record — verify against primary for final citation

#### Zumes - "Non-violent action against apartheid" (SA History Online)
**Type:** Historical analysis
**Domain:** Anti-Apartheid History
**Rating:** A
**Year:** undated
**Relevance:** Nonviolent resistance arc: Defiance Campaign → exile → UDF → sanctions+strikes forcing negotiation
**Used In:** organizing-historical-campaigns.md

#### Hobson - Stonewall NPS Scholars' Workshop essay
**Type:** Academic historical context report
**Domain:** LGBTQ+ History
**Rating:** A
**Year:** 2016
**Relevance:** Stonewall complexity (multi-day uprising, contested narratives, GLF/STAR/GAA differentiation)
**Used In:** organizing-historical-campaigns.md

#### Fossil-Fuel Divestment Effectiveness Systematic Review (WIREs Climate Change, 2024)
**Type:** Peer-reviewed systematic review (69 articles)
**Domain:** Climate Policy / Social Movements
**Rating:** A
**Year:** 2024
**Relevance:** Divestment real effects = social-license erosion + agenda-setting, contested on capital effects; 3-order impact framework
**Used In:** organizing-historical-campaigns.md

#### Married Women's Property Acts scholarship (Mississippi Encyclopedia; Emerald enactment study; LoC guides)
**Type:** Institutional/academic histories
**Domain:** Legal History, Women's Rights
**Rating:** A
**Year:** various
**Relevance:** MS 1839 first MWPA driven by debt-shielding (elite interest) not feminism; Betsy Love Allen Chickasaw precedent; NY 1848 as template
**Used In:** organizing-historical-campaigns.md

#### Women's ordination timeline sources (Encyclopedia.com, Andrews U, LoC)
**Type:** Academic/institutional histories
**Domain:** Religious History, Women's Rights
**Rating:** B+/A
**Year:** various
**Relevance:** Women preached first in marginal/revival movements (Quakers, Holiness, Pentecostal), absorbed by mainlines 1956-1980
**Used In:** organizing-historical-campaigns.md

#### UK Undercover Policing Inquiry + Mark Kennedy case reporting
**Type:** Public inquiry / investigative journalism
**Domain:** Policing, Infiltration
**Rating:** A
**Year:** 2010-ongoing
**Relevance:** Modern provocateur infiltration (7+ yr deep cover, relationship deception, protest provocation)
**Used In:** organizing-countertactics.md

#### False-flag primary record (Nuremberg/Gleiwitz, declassified NSA/Tonkin, Soviet archives/Mainila, Inkathagate/TRC)
**Type:** Declassified primary documents
**Domain:** Intelligence/False Flag History
**Rating:** A+
**Year:** various
**Relevance:** Confirmed manufactured-pretext cases + crisis exploitation pattern; Inkatha covert funding as controlled opposition
**Used In:** organizing-countertactics.md

---

## Source Gaps to Fill

### High Priority
1. **Community Organizing** - FILLED 2026-10-31 (Alinsky/Ganz/McAlevey/Han from sprint reviews; Chenoweth, Church Committee, case histories added)
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
