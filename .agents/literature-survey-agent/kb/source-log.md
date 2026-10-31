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

### Round 2: Enforcement, Campaign Masters, Modern Playbook, Info Infrastructure (added 2026-10-31)

#### The Intercept/Grist - TigerSwan Standing Rock document leaks (2017, 2023)
**Type:** Investigative journalism on leaked primary documents (100+ leaked + 1,000+ public-records docs)
**Domain:** Private Security, Counter-Insurgency
**Rating:** A
**Year:** 2017-2023
**Relevance:** Fullest documented private counter-organizing operation: camp infiltration, "jihadist insurgency" framing, fake reporters, police liaison/fusion, social-media monitoring, $175k fine as cost-of-business
**Used In:** organizing-modern-counter-playbook.md

#### HRW "Foreign Agent Laws in the Authoritarian Playbook" (2024) + CSCE/ICNL trackers
**Type:** NGO research + government briefing
**Domain:** Authoritarianism, Civil Society Law
**Rating:** A
**Year:** 2024
**Relevance:** Russia 2012 archetype → proliferation (Georgia, Kyrgyzstan, Hungary, Slovakia); stigmatize + administrative-death + funding-cut mechanism
**Used In:** organizing-modern-counter-playbook.md

#### Citizen Lab Pegasus investigations + Pegasus Project (Forbidden Stories)
**Type:** Forensic research lab + journalism consortium
**Domain:** Digital Surveillance
**Rating:** A+
**Year:** 2021-2024
**Relevance:** Forensically confirmed spyware infections of activists/journalists; zero-click era of surveillance = subscription, not field agent
**Used In:** organizing-modern-counter-playbook.md

#### APSR antislavery petition dataset study (2014) - "When Canvassers Became Activists"
**Type:** Peer-reviewed empirical study (8,500+ petitions, 1833-1845)
**Domain:** Political Science, Movement History
**Rating:** A+
**Year:** 2014
**Relevance:** Petition canvassing = leadership pipeline (women 50%+ more effective; Seneca Falls signers were canvassers; 4x female overrepresentation among later activists)
**Used In:** organizing-information-infrastructure.md

#### House History / National Archives - Gag Rule records
**Type:** Primary government records
**Domain:** Abolition, Congressional History
**Rating:** A+
**Year:** 1836-1844 materials
**Relevance:** 130k+ petitions; gag-rule suppression backfired into the movement's best story (deliberate proto-Streisand)
**Used In:** organizing-information-infrastructure.md

#### Tufekci - *Twitter and Tear Gas* (2017)
**Type:** Academic book
**Domain:** Digital Movements, Sociology
**Rating:** A
**Year:** 2017
**Relevance:** "Tactical freeze" — networked movements scale attention faster than org capacity; mobilization≠organizing
**Used In:** organizing-information-infrastructure.md

#### Popovic - *Blueprint for Revolution* + Otpor/CANVAS documentation + Color Revolution scholarship
**Type:** Practitioner memoir + academic scholarship
**Domain:** Nonviolent Resistance, Movement Export
**Rating:** B+/A
**Year:** 2000-2015
**Relevance:** Laughtivism, arrest-training, brand-based participation, CANVAS export model and its failure modes vs unrestrained regimes (Belarus, Iran)
**Used In:** organizing-campaign-masters.md

#### 504 sit-in documentation (Smithsonian, NPS, Longmore Institute oral histories, DSQ "Lomax's Matrix")
**Type:** Institutional history + oral history + peer-reviewed
**Domain:** Disability Rights
**Rating:** A/A+
**Year:** 1977 event, documented 2000s+
**Relevance:** Coalition matrix logistics (Panther food line), HEW phone-cut workaround, unspinnable optics, signed-unchanged win, 504→ADA continuity
**Used In:** organizing-campaign-masters.md, organizing-enforcement-and-defense.md

#### Energy Transfer v. Greenpeace verdict (2025) + Cop City RICO indictments (2023-)
**Type:** Court records
**Domain:** Lawfare, Counter-Organizing
**Rating:** A
**Year:** 2023-2025
**Relevance:** Civil damages + terrorism charges as counter-movement weapons; bail funds/infra as targets (Greenpeace appeal pending — contested)
**Used In:** organizing-modern-counter-playbook.md, organizing-enforcement-and-defense.md

#### Enforcement-phase historiography (Foner Reconstruction; Shelby County record; ALRB capture; anti-abortion infrastructure scholarship)
**Type:** Academic histories + court records
**Domain:** Post-Victory Movement Dynamics
**Rating:** A/A+
**Year:** various
**Relevance:** Movements disband, losers reorganize; enforcement-form tiers; agency capture; 49-year counter-movement win
**Used In:** organizing-enforcement-and-defense.md

---

### Round 3: Anti-Colonial, Messianic, Pre-Modern, Secret Societies (added 2026-10-31)

#### Norman Cohn - *The Pursuit of the Millennium* (1957/1970)
**Type:** Academic book (Oxford UP)
**Domain:** Medieval Millenarianism, Religious Movements
**Rating:** A
**Year:** 1957/1970
**Relevance:** The spine for prophetic mobilization: prophet class (lower clergy/marginal intellectuals), constituency (rootless poor), narrative structure (golden age→corruption→imminent overturn), cases from Niklashausen to Münster
**Used In:** organizing-messianic-prophetic.md
**Notes:** Still the only comprehensive work. Cohn's modern-totalitarianism linkage is contested; case history is standard.

#### Emmanuel Le Roy Ladurie - *Montaillou* (Fournier register) + Bernard Gui - *Practica Inquisitionis*
**Type:** Inquisitorial primary records / microhistory + inquisitor's manual
**Domain:** Medieval Counter-Intelligence, Heresy
**Rating:** A+
**Year:** 1318-25 register; manual 1320s
**Relevance:** Best-documented pre-modern counter-organizing institution: dossier system, informer networks, grace-period amnesty harvests, secret denunciation, relapse=death engineered informer class
**Used In:** organizing-premodern-resistance.md

#### Haitian Revolution scholarship (Geggus; Bois Caiman ceremony analyses)
**Type:** Academic monograph + journal articles
**Domain:** Slave Revolt, Caribbean History
**Rating:** A
**Year:** various
**Relevance:** Commandeur network pre-organization, Vodou as cross-ethnic binding infrastructure, blood oath as commitment device, conspirators' fake-decree disinfo document
**Used In:** organizing-anticolonial-campaigns.md
**Notes:** Ceremony details contested; consensus on religious-network organizational function.

#### Land League scholarship (Parnell Ennis speech — primary; BAHS organizational study)
**Type:** Primary speech + peer-reviewed history
**Domain:** Irish History, Agrarian Movements
**Rating:** A
**Year:** 1879-82 events
**Relevance:** Boycott invention ("moral Coventry"); boycotting primarily as insider discipline; Ladies' Land League pre-built succession redundancy
**Used In:** organizing-anticolonial-campaigns.md

#### Battle of Algiers / counter-insurgency studies (DTIC French HUMINT study; RAND *Pacification in Algeria*; J Cold War Studies on guerre revolutionnaire; Ormsby on Galula)
**Type:** Military-academic analysis
**Domain:** Counter-Insurgency Doctrine
**Rating:** A
**Year:** various
**Relevance:** Two-register COIN (mass brutal vs surgical bleuite); torture's strategic self-defeat; Galula hearts-and-minds paradox; COIN as competitive organizing
**Used In:** organizing-anticolonial-campaigns.md

#### Waldensian/Hussite/Remenca scholarship (Biller *The Waldenses*; Kaminsky; Freedman; Cohn *Lust for Liberty*)
**Type:** Academic monographs
**Domain:** Medieval Religious Resistance
**Rating:** A
**Year:** various
**Relevance:** Clandestine parallel-church durability (800yr), wagon-fort tactical innovation, Compactata negotiated win + moderate-split exploitation, rare peasant negotiated victory
**Used In:** organizing-premodern-resistance.md

#### Secret society scholarship (Rath on Carbonari AHR 1964; Kershaw/Rice on Mau Mau oath; ter Haar on Triads; Fenian ritual documents; Le Caron account)
**Type:** Peer-reviewed + primary documents
**Domain:** Clandestine Organizations
**Rating:** A
**Year:** various
**Relevance:** Oath as costly-signal commitment tech; staged degrees; cover functions; Le Caron 25-year penetration; counter-oathing ceremonies; dual-org architecture
**Used In:** organizing-secret-societies.md
**Notes:** Le Caron is the infiltrator's own account — self-serving but primary.

---

#### Jasper & Poulsen, "Recruiting Strangers and Friends: Moral Shocks and Social Networks," Social Problems 42(4)
**Type:** Peer-reviewed survey study (animal-rights & anti-nuclear protestors)
**Domain:** Recruitment sociology
**Rating:** A+ (~760 citations; the stranger-recruitment paper)
**Year:** 1995
**Relevance:** Two recruitment channels — network activation vs moral shocks to strangers via condensing symbols
**Used In:** organizing-prisons-and-release.md

---

#### Suffragette prison record (UK Parliament heritage pages; London Museum; OpenLearn)
**Type:** Institutional/primary-source histories
**Domain:** Political imprisonment / hunger strikes
**Rating:** A/A+
**Year:** events 1909–1914
**Relevance:** Dunlop's 1909 hunger strike (status demand), force-feeding as propaganda gift, Cat and Mouse Act failure mode
**Used In:** organizing-prisons-and-release.md

---

#### Attica Liberation Faction manifesto + historiography (Freedom Archives primary; Race & Class 2011; Harvard CR-CL; NYS Archives)
**Type:** Primary document + peer-reviewed history
**Domain:** Prison organizing
**Rating:** A
**Year:** events 1971
**Relevance:** 28 "strikingly moderate" demands; manifesto-drafting as organizing; amnesty as the state's real refusal point; jailhouse-lawyer strand
**Used In:** organizing-prisons-and-release.md

---

#### Amnesty International archive (AI@50 timeline; 1981 membership handbook; Urgent Action records)
**Type:** Primary institutional records
**Domain:** Release-campaign architecture
**Rating:** A
**Year:** 1961–present
**Relevance:** Adoption groups (prisoner-as-charter); "one letter is not pressure" volume doctrine; elite-statement prestige multiplier; response latency
**Used In:** organizing-prisons-and-release.md

---

### Round 3.5: Faith-Power Interface & Critical Pedagogy (added 2026-10-31)

#### Niebuhr - *The Social Sources of Denominationalism* (1929); Weber/Troeltsch church-sect frame
**Type:** Academic books (sociology of religion canon)
**Domain:** Sociology of Religion
**Rating:** A
**Year:** 1909-1929
**Relevance:** Co-option by institutionalization: sects of the disinherited → denominations → churches; routinization of charisma as the mechanism
**Used In:** organizing-faith-and-power.md
**Notes:** Niebuhr's evolutionism is dated; the dynamic (respectability drift) remains standard.

#### Foucault - *Discipline and Punish*, *Madness and Civilization*, pastoral power lectures
**Type:** Academic books/lectures
**Domain:** Institutional Power, Surveillance
**Rating:** A+
**Year:** 1961-1980
**Relevance:** Sovereign vs disciplinary power; examination as power tech (confession→dossier lineage); Great Confinement as institution for surplus population; carceral continuum; Panopticon internalization
**Used In:** organizing-faith-and-power.md, organizing-critical-pedagogy.md

#### Genovese - *Roll Jordan Roll* (1974) + driver scholarship (Starobin revisionist school)
**Type:** Academic history (Gramscian)
**Domain:** Slavery Studies
**Rating:** A
**Year:** 1974+
**Relevance:** Paternalism as contested bargain (slaves weaponized the master's frame); driver as structurally dual intermediary
**Used In:** organizing-faith-and-power.md
**Notes:** Paternalism thesis contested but generative; driver-as-mediator is the revisionist consensus.

#### Slave Bible documentation (Glasgow special collections; Porteus correspondence)
**Type:** Primary artifact + institutional scholarship
**Domain:** Missionary History, Information Control
**Rating:** A
**Year:** 1807 artifact
**Relevance:** Literal redacted scripture (90% OT/50% NT cut; Exodus/Psalms/Gal 3:28/Revelation removed) — faith-as-information-weapon, documented
**Used In:** organizing-faith-and-power.md

#### Mission-education/comprador scholarship (Attwell intermediary intellectuals; Macaulay minute; colonial communal electorates)
**Type:** Academic articles + primary documents
**Domain:** Colonial Education, Intermediary Classes
**Rating:** A
**Year:** various
**Relevance:** Mission schools produced the intermediary class → co-option backfire (Nkrumah/Mandela/Kenyatta mission-schooled)
**Used In:** organizing-faith-and-power.md

#### Freire - *Pedagogy of the Oppressed* (1970); Boal - *Theatre of the Oppressed* (1979)
**Type:** Canonical movement-education texts
**Domain:** Critical Pedagogy
**Rating:** A+
**Year:** 1970/1979
**Relevance:** Banking vs problem-posing; conscientization; praxis loop; spect-actor rehearsal; Joker facilitation
**Used In:** organizing-critical-pedagogy.md

#### Horton & Freire - *We Make the Road by Walking* (1990); Highlander historiography (Morris "halfway house"; Citizenship School records)
**Type:** Primary dialogue + academic history
**Domain:** Movement Education
**Rating:** A
**Year:** 1932-1990
**Relevance:** The US practitioner precedent; community-teachers-only rule (Robinson case ~80% voter registration); movement halfway house as institution type
**Used In:** organizing-critical-pedagogy.md

#### Illich - *Deschooling Society*; Bourdieu & Passeron - *Reproduction*; Gramsci - *Prison Notebooks*
**Type:** Canonical critical-institutional texts
**Domain:** Institutional Critique, Hegemony
**Rating:** A+
**Year:** 1929-1971
**Relevance:** Institutional education as domestication (Illich); symbolic violence/cultural capital (Bourdieu); organic intellectuals + war of position (Gramsci)
**Used In:** organizing-critical-pedagogy.md

#### Gutiérrez - *A Theology of Liberation* (1971) + BEC scholarship; Fals-Borda PAR; hooks; Rancière - *The Ignorant Schoolmaster*
**Type:** Canonical + academic
**Domain:** Liberation Theology, PAR, Critical Pedagogy
**Rating:** A/A+
**Year:** 1971-1994
**Relevance:** BECs as Freirean praxis in church form; research-as-organizing; engaged pedagogy; Rancière counter-point (equal intelligence axiom vs teacher-led liberation)
**Used In:** organizing-critical-pedagogy.md, organizing-faith-and-power.md

---

### History of Technology / Ink Making

#### Cennini - Il Libro dell'Arte ch. XCI (c.1390s)
**Type:** Primary source (craft handbook)
**Domain:** Medieval craft technology
**Rating:** A+
**Relevance:** Linseed oil boiled in glazed pot on enclosed stove; flame must not reach oil; "risk burning down the house"; reduce by half; mordants and tempera
**Used In:** ink-oil-boiling.md lit review

#### Encyclopedie (Diderot & d'Alembert) - "Encre noir" (1751-72)
**Type:** Primary technical encyclopedia
**Domain:** Print history / material culture
**Rating:** A
**Relevance:** Full printing-ink recipe: oil heated to varnish, deliberate ignition smothered with wet linen, 1lb bread crusts + dozen onions to degrease, slate-drip readiness test, "wide space such as a garden" siting, sac-a-noir soot chamber
**Used In:** ink-oil-boiling.md lit review

#### NCBI - Printing Processes and Printing Inks
**Type:** Scholarly book chapter (conservation science)
**Domain:** History of printing
**Rating:** A
**Relevance:** Oil-based ink required for metal type; lampblack + heated linseed/nut oil + rosin + metal driers; heating was "significant hazard (fires, vapours, spills)"
**Used In:** ink-oil-boiling.md lit review

#### Savage, Stiber & Morenus - Pre-Industrial Western Printing Inks c.1450-1850
**Type:** Scholarly survey chapter (SAS-Space)
**Domain:** Print history
**Rating:** A
**Relevance:** Most substantial survey of pre-industrial ink materiality; recipe secrecy until ~17th c.; Dutch vs English varnish practice
**Used In:** ink-oil-boiling.md lit review

#### "The Path to Pistoia" - Past & Present 246 (OUP)
**Type:** Peer-reviewed journal article
**Domain:** Urban environmental history
**Rating:** A
**Relevance:** Pistoia statute banishing stench-crafts beyond the walls; urban zoning of nuisance trades
**Used In:** ink-oil-boiling.md lit review

#### Fire Prevention in Late Medieval British Towns (Oxford Academic)
**Type:** Scholarly book chapter
**Domain:** Urban history / fire regulation
**Rating:** A
**Relevance:** Towns regulated furnace/oven crafts and combustibles; fire-fighting bylaws, penalties for those responsible
**Used In:** ink-oil-boiling.md lit review

#### Henley's 20th Century Formulas + NIST Circular 53
**Type:** Trade encyclopedias / government circular
**Domain:** Ink manufacture technology
**Rating:** B/C
**Relevance:** Boil-over and acrolein fume hazards, fire-on-rails safety rigs, "long" vs "short" varnish drip test, red-hot poker ignition
**Used In:** ink-oil-boiling.md lit review

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
