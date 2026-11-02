# Victorian Diseases Literature Review

**Survey Date:** 2026-10-31
**Agent:** Literature Survey Agent
**Confidence:** High

## Summary

A big research pass on the disease landscape of Victorian Britain (1837–1901) — the last
great era in which infectious disease dominated mortality (~40–45% of all deaths, >50% in
towns). Covers the epidemiological profile, the major diseases with their period names,
transmission routes and seasonality, the sanitation and germ-theory revolutions that ended
them, and the social/organizing dynamics (blame, compulsion, anti-vaccination campaigns)
that disease produced. Ends with concrete translations into the existing `illness.rs`
compartment model and a naming table for in-game diseases.

This is the era where disease meets our game's other obsessions: organizing (the
anti-vaccination leagues are a canonical resistance campaign), blame and grievance (the
ergot bread-blame mechanic already in `session.rs` has Victorian twins), and infrastructure
as salvation (sewers beat sermons).

## Key Sources

### Mooney, G. (2007) — "Infectious Diseases and Epidemiologic Transition in Victorian Britain? Definitely" (*Social History of Medicine*)
**Rating:** A+
**Relevance:** Authoritative answer to "how much did infection kill?" — ~40% of deaths nationally in the 1850s, >50% in towns/cities where most people lived.
**Key Findings:**
- Infectious disease was the dominant mortality regime until late in the century.
- Disaggregation by place/age/sex matters: the "disease environment" was intensely local.

### Floud/Harris-style GRO data — *Annual Deaths by Cause, England and Wales 1848–1900* (UK Data Archive SN 5705) + PMC "history of infectious diseases" overview
**Rating:** A
**Relevance:** Hard numbers for the disease roster.
**Key Findings (England & Wales, 1850):**
- Infectious diseases: 44.7% of all deaths (26.2% non-respiratory, 18.5% respiratory).
- Between 1848–1872, **tuberculosis was the leading cause of death: 15% of ALL deaths.**
- Bronchitis 6.7%, scarlet fever + diphtheria 5.7%, diarrhoea + enteritis 4.4%.
- Heart disease + stroke only 5.8%; cancer 1.7% — people died too young for modern killers.

### Snow, J. (1849/1854) — *On the Mode of Communication of Cholera* + Brody et al., *Lancet* "Map-making and myth-making in Broad Street" (2000)
**Rating:** A+ (primary + peer-reviewed historiography)
**Relevance:** The canonical waterborne-transmission story; how disease investigation works.
**Key Findings:**
- Cholera struck Britain four times: 1831–32, 1848–49, 1853–54, 1866.
- Snow's real contribution wasn't the map — it was the "natural experiment" (Southwark & Vauxhall vs Lambeth water companies): ~two-thirds of the dead would have lived with cleaner water.
- The folk-tale version (Snow yanks the pump handle, epidemic stops) is apocryphal — the outbreak was already waning. Good design caution: legends > data in public memory.
- Snow coined "mode of communication" because "contagion" implied touch-borne only — cholera was faecal-oral via water, a *different route*.

### The Science Museum — "Cholera in Victorian London"
**Rating:** A- (museum scholarship, well-sourced)
**Relevance:** The miasma-vs-contagion frame that shaped ALL Victorian health policy.
**Key Findings:**
- Miasmatists cleaned filth (helped anyway!); contagionists quarantined (mostly failed for waterborne disease).
- Chadwick's 1842 *Sanitary Conditions of the Labouring Population* → Public Health Act 1848 (non-compulsory → mostly ignored). Chadwick's sewer-flushing accidentally *worsened* cholera by putting more raw sewage into the Thames water supply.
- *The Times*, 1848: cholera was "the best of all sanitary reformers" — epidemics created political will.

### London Museum / BBC — The Great Stink (1858) & Bazalgette's sewers
**Rating:** A-
**Key Findings:**
- Hot summer + low Thames + Parliament choking on smell → emergency legislation in weeks.
- Bazalgette: 1,100 miles of drains + 82 miles of brick sewers + intercepting sewers, completed 1875, still in use. Infrastructure as the actual cure.
- Sanitary investment peaked 1890–1900 → dramatic infant mortality decline.

### Hardy, A. — *The Epidemic Streets* (Oxford, 1993)
**Rating:** A+
**Relevance:** The standard academic work on Victorian infectious disease behavior.
**Key Findings:**
- Disease-by-disease epidemiology: whooping cough, measles, scarlet fever, diphtheria, smallpox, typhus, typhoid, TB.
- Childhood diseases were *crowding*-sensitive: needed a "regularly replenished population of susceptibles" — a village with few kids gets epidemic spikes, not endemic smoulder.
- Prevention (vaccination, notification, isolation) beat therapy — Victorian medicine could barely treat, only prevent.

### "Urban Famine or Urban Crisis? Typhus in the Victorian City" (*Medical History*)
**Rating:** A
**Key Findings:**
- Typhus = the poverty disease: 93% of Liverpool's typhus deaths among weekly-wage earners in courts/alleys "where sanitary provisions are conspicuously absent."
- Louse-borne; thrived on overcrowding, cold winters, doubling-up when wages failed. Seasonal economy → crowding → typhus is a causal chain the game can model.
- Period names: "putrid fever", "camp fever", "ship fever", "jail fever", "Irish fever" (blamed on famine migrants).

### "Sex differentials in phthisis mortality" (*Continuity and Change* 2015) + PMC "Trends in tuberculosis in the UK"
**Rating:** A
**Key Findings:**
- Phthisis/consumption = single most important cause of death mid-century; ~1 in 4 deaths early 19th c., 15% of all deaths 1848–72, 11% in Leeds at century's end.
- Notably a killer of young adults, especially women 15–40 in rural areas (return-migration of the sick, intra-household nutrition bargaining debated).
- Long wasting course — "consumption" = the body consuming itself; romanticized in fiction precisely because it was slow and visible.
- Case fatality ~50% pre-chemotherapy; declined ~3.3%/yr after ~1860 on nutrition/housing alone — no cure existed.

### "The Filth Disease" — typhoid historiography + Lancet "Beyond Bazalgette"
**Rating:** A
**Key Findings:**
- Typhoid killed Prince Albert (1861); nearly killed the Prince of Wales (1871, at Londesborough Lodge — "placed at the summit of a great length of sewer"). Rank was no shield — the era's best argument for public goods.
- Typhoid investigation *created* modern epidemiology; reformers weaponized it to sell sewer bonds.
- Water- and food-borne; "slow fever" — 3–4 week course, ~10–20% fatal.

### Porter & Porter / Fitzpatrick — anti-vaccination politics (*Medical History* + Lancet primary)
**Rating:** A
**Key Findings:**
- Vaccination timeline: Jenner 1796 → free infant vaccination 1840 → **compulsory 1853** → stricter 1871 after the 1870–73 epidemic.
- Backlash produced the **Anti-Compulsory Vaccination League** — the era's most successful single-issue resistance campaign; conscience-clause victory 1898/1907.
- **The Leicester Method**: Leicester's anti-vaccinationists built an *alternative* system — notification → isolation → quarantine of contacts → disinfection → (later) compensated quarantine — that worked WITHOUT compulsion. A perfect game model of "the movement proposes its own institution."
- Parallel: Contagious Diseases Acts (compulsory screening of prostitutes in garrison towns) → repeal campaign led by Josephine Butler — disease law as liberty fight.

### Gale et al. — "A century of changes in mortality of principal infections of childhood" + Hardy "Rickets and the Rest" (*Social History of Medicine*)
**Rating:** A
**Key Findings:**
- Scarlet fever: the great child-killer of mid-century (peak virulence ~1840–70), then a mysterious steep decline — still not fully explained.
- Diphtheria: rose 1850s+, "the strangling angel" — death by airway membrane; period synonyms *quinsy*, *laryngitis* often meant diphtheria.
- Measles + whooping cough: endemic child diseases, rarely fatal over age 5, fatality amplified by malnutrition/rickets.
- Rickets: "the English disease" — coal-smoke darkness + diet; peaked in deaths late winter/spring (vitamin D deprivation), and its subtle role was *amplifying other diseases' fatality* — a comorbidity multiplier.
- "Teething" on death registers often meant infant diarrhoea — Victorian euphemism in the stats themselves.

### CampOP "Shitscapes" paper — infant diarrhoea
**Rating:** A
**Key Findings:**
- Infant diarrhoeal mortality **did not decline until after 1900** — the one big failure of Victorian sanitation; peaked in the hot summers of the 1890s (1911 heatwave → massive epidemic).
- Water improvements fixed cholera/typhoid/dysentery but NOT infant diarrhoea — it was fly/food/milk-borne. Different route → different fix. A design lesson: one sanitation upgrade shouldn't cure everything.

### "Fenland ague in the nineteenth century" (*Medical History*)
**Rating:** A
**Key Findings:**
- England had endemic malaria: tertian ague (*P. vivax*) in the Lincolnshire/Norfolk/Suffolk/Cambridge fens, epidemic 1826–29 and 1857–60.
- High morbidity, low mortality, **seasonal relapse** — the signature is recurrence, not death; depressed labor and amplified co-morbidity for decades.
- Folk names: marsh fever, autumnal fever, the shakes, "the Bailiff of the Marshes", "Old Johnny Axey" — disease as local character.

### Occupational disease sources — phossy jaw (USURJ), Pott/chimney sweeps (*Occupational & Environmental Medicine*), ILO history
**Rating:** A/B+
**Key Findings:**
- **Phossy jaw**: white-phosphorus necrosis in match workers (Bryant & May, Bow). Jaw bone rotted, glowed greenish-white; the 1888 matchgirls' strike is a canonical labor-organizing story — *disease → organizing* in one.
- **Chimney sweep's cancer**: Pott 1775, first occupational cancer ever identified; 29% of London sweep deaths from scrotal cancer into the 19th c.
- **Miner's phthisis / "the dust"**: pneumoconiosis; **byssinosis**: cotton-dust "Monday fever"; **mad hatter**: mercury tremor; lead poisoning in potters/painters; arsenic in match/aniline trades.
- Occupational disease is *vocation*-keyed — maps directly onto the game's `Vocation` enum.

### Valtierra et al. (PNAS 2010) — the 1889 "Russian flu"
**Rating:** A
**Key Findings:**
- 1889–94 pandemic: first industrial-era pandemic — spread by rail and sea, circled the globe in ~4 months, R0 ≈ 2.1, attack rate ~60%, CFR 0.1–0.28%.
- Possibly NOT influenza (HCoV-OC43 coronavirus hypothesis) — but period name is what matters: **"la grippe"**, "the Russian flu".
- Relapses were famous — weakness lingered after fever broke.

### Farr's "statistical nosology" — Registrar-General classification (1842)
**Rating:** A+ (primary)
**Key Findings:**
- William Farr's five classes: **Zymotic** (epidemic/endemic/contagious — "caused by a ferment in the system"), Constitutional (cachectic), Local (monorganic), Developmental (metamorphic), Violent (thanatic).
- "Zymotic" is a lovely period-correct tag for communicable disease — usable as our disease-class enum name.
- Farr also documented Victorian euphemisms: 'typhoid/typhus' conflated; 'quinsy'/'laryngitis' = diphtheria; 'teething' = infant diarrhoea; 'puerperal fever' = strep A; 'hydrocephalus' = tubercular meningitis.

### PubMed 20027740 — historical disease terminology, Victoria 1853–1900
**Rating:** A
**Key Findings:** Systematic mapping of period names → modern diagnoses (see naming table below).

## The Disease Roster

| Period name | Modern name | Route | Season | Character | Victorian signature |
|---|---|---|---|---|---|
| consumption / phthisis | tuberculosis | droplet, chronic | none (worse winter) | slow wasting, years | #1 killer; romanticized; young adults |
| the cholera / Asiatic cholera / cholera morbus* | cholera | water, faecal-oral | summer | dies in days, rice-water stools | terror + sanitary reform driver |
| typhoid / "the filth disease" / slow fever | typhoid | water/food, faecal-oral | autumn | 3–4 wk fever, rash | killed the Prince Consort |
| typhus / putrid fever / camp fever / ship fever / jail fever | typhus | body lice | winter | acute high fever, delirium | poverty's disease; 93% working-class deaths |
| scarlatina / canker rash | scarlet fever | droplet | winter/spring | rash + sore throat, children | the great child-killer, 1840–70 |
| diphtheria / quinsy / "the strangling angel" | diphtheria | droplet | winter | throat membrane, suffocation | rose as scarlet fever waned |
| smallpox / the speckled monster | smallpox | droplet/contact | winter | pocks, ~30% fatal | vaccination + anti-vax war |
| measles | measles | droplet | spring | childhood, "harmless" until it wasn't | fatality tracked poverty |
| whooping cough / chin-cough | pertussis | droplet | winter/spring | infant killer | "teething" cover-name |
| la grippe / Russian flu | influenza(?) | droplet | winter | fast, high attack rate, low CFR | 1889–94; relapses famous |
| ague / marsh fever / the shakes | malaria (P. vivax) | mosquito | autumn | recurring paroxysms, years | Fenland endemic; low kill, high drag |
| childbed fever / puerperal fever | strep A sepsis | doctors' hands | none | post-partum, days | "the doctor's plague"; Semmelweis |
| summer diarrhoea | infant diarrhoea (mixed) | flies/milk/food | summer heat | infant killer | survived everything Victorian reformers threw at it |
| phossy jaw | phosphorus necrosis | occupational | n/a | jaw rot, glows green | match girls → 1888 strike |
| miner's phthisis / the dust | pneumoconiosis | occupational | n/a | breath shortens over years | vocation-keyed |
| sweeps' cancer | scrotal SCC | occupational | n/a | first identified occupational cancer | climbing boys |
| rickets / the English disease | vitamin D deficiency | environment | late winter/spring | bowed bones; *amplifies other killers* | coal smoke stole the sun |
| ergotism / St Anthony's fire | ergot poisoning | food (rye) | damp harvest years | burning limbs, convulsions | nearly gone by Victoria's reign — but lived on in folk memory |
| brain fever | meningitis/typhus | varies | — | delirium | literary staple |
| dropsy / apoplexy / blood poison | oedema / stroke / sepsis | non-infectious | — | symptom-labels | Farr's nosology names |

*cholera morbus strictly meant English summer diarrhoea before 1831; Victorians kept using it loosely.

## Game Mechanic Proposals

### Mechanic 1: Zymotic Roster — diseases as data, not strings
**Academic Basis:** Farr's nosology + the roster above; illness.rs already has a `name: String`.
**Translation:**
- A static `DiseaseKind` table: { name, period_name, contagiousness, route (contact | waterborne | vector | occupational | foodborne | noninfectious), virulence (severity growth), lethality, immunity_conferred, incubation_ticks, season }.
- Everything currently special-cased on the string "flu"/"the fever" looks up the kind instead.
**Educational Value:** Players learn which diseases exist and how they differ — cholera ≠ flu.
**Fun Factor:** Outbreaks become identifiable threats with counterplay, not generic damage.
**Implementation:** `game2-core/src/sim/illness.rs` — new `DiseaseKind`/`Zymotic` enum + `Disease.kind` field; keep `name` for display.

### Mechanic 2: Incubation (the already-existing `Stage::Latent`)
**Academic Basis:** Victorian incubations: cholera 12h–5d, measles ~12d, scarlet 2–5d, smallpox ~12d, typhoid 1–3wk.
**Translation:**
- New cases start `Stage::Latent` with a per-kind `incubation` timer; during latency they don't damage and don't shed (or shed weakly — cholera sheds during incubation, measles doesn't until prodrome).
**Educational Value:** Why quarantine windows existed; why a healthy-looking traveller is scarier than a sick one.
**Fun Factor:** "Who else caught it?" tension; isolation decisions under uncertainty.
**Implementation:** `Disease.incubation_left: u32`; `stage_for` keeps severity logic; spread_contagion skips `Latent` unless kind sheds.

### Mechanic 3: Waterborne route — the Broad Street mechanic
**Academic Basis:** Snow 1849/54; Southwark & Vauxhall natural experiment; the game ALREADY tracks `pit_water` and per-tile `effluent` (session.rs ~2200) — the sim has a Thames-in-miniature.
**Translation:**
- Sick villager with a waterborne kind → effluent deposited at their home tile / pit.
- Villagers drawing water from contaminated tile → exposure roll independent of proximity.
- Cleaning/boiling water or fixing the pit breaks the chain — infrastructure as cure.
**Educational Value:** THE core Victorian insight: the well, not the neighbour, was the enemy.
**Fun Factor:** Player can literally find-the-pump: "which tile is sickening everyone?"
**Implementation:** reuse `effluent: HashMap<Tile>`; waterborne kinds contribute effluent; drinking/food goals check tile cleanliness.

### Mechanic 4: Seasonality — the fever calendar
**Academic Basis:** summer diarrhoea/cholera (hot months), typhus/influenza/bronchitis (cold months), ague (autumn), rickets deaths (late spring), ergot (damp harvest years — already `tick % 365 == 210` gated!).
**Translation:**
- Per-kind `season: Option<(start_tick, end_tick)>` on the year-cycle; outbreak spawners roll within season.
- Heat waves boost diarrhoeal kinds (1890s hot-summer epidemics; 1911).
**Educational Value:** Why Victorians feared August, not January, for the belly diseases.
**Fun Factor:** Predictable dread — "the fever season is coming, stockpile herbs."
**Implementation:** FLU_DAY / ergot gates already pattern this; generalize into a `seasonal_outbreaks` table.

### Mechanic 5: Differential immunity & relapse
**Academic Basis:** smallpox = durable immunity; influenza = weak/short; ague = relapse after dormancy (vivax hypnozoite); TB = latent reactivation under malnutrition.
**Translation:**
- `immunity_conferred` per kind (smallpox 0.9, influenza 0.2, cholera 0.4).
- Relapsing kinds leave a dormant `Relapse` component: ague fires again each autumn; TB reactivates if Hunger stays high.
**Educational Value:** Why some diseases "visited once" and others never left.
**Fun Factor:** Consumption as a slow-burn tragedy thread; the ague-wife who fails every harvest.
**Implementation:** extend `ACQUIRED_IMMUNITY` to per-kind; `Relapse { kind, next_due }`.

### Mechanic 6: Crowding & class — typhus respects the rent
**Academic Basis:** Liverpool typhus data (93% wage-earner deaths); childhood diseases "regularly replenished susceptibles"; overcrowding as THE cited cause.
**Translation:**
- Exposure scales by occupants-per-dwelling, not just distance: same-house neighbours get a multiplier.
- Villager `weary`/hunger state lowers effective immunity (malnutrition → disease, the Victorian feedback loop).
**Educational Value:** Why the same germ killed the poor more — poverty IS a transmission route.
**Fun Factor:** Tenement upgrades and wage relief become *health policy*.
**Implementation:** `spread_contagion` already uses `Pos`; add household co-location bonus + immunity debuff from `Hunger`.

### Mechanic 7: Occupational kinds — the vocation tax
**Academic Basis:** phossy jaw/match girls, sweep's cancer, miner's phthisis, byssinosis.
**Translation:**
- Non-contagious kinds keyed to `Vocation`: long service at a trade accrues `trade_exposure`; crossing a threshold inflicts the trade's disease (chronic, incurable-ish, reduces skill).
- Match girls → "the phossy jaw"; miners → "the dust"; sweeps → "sweep's rot".
**Educational Value:** Industrial disease was invisible for decades — "it was just toothache."
**Fun Factor:** Guild reform vs output tension; the strike writes itself (Bryant & May 1888).
**Implementation:** `Craft.exposure` field + threshold check in vocation tick.

### Mechanic 8: Blame & remedy — miasma-minded villagers
**Academic Basis:** ergot bread-blame (implemented!); cholera blamed on migrants/Irish/railways; "fever nests"; Snow's legend > his data.
**Translation:**
- During outbreaks, villagers generate `Grievance::Blamed` toward plausible-but-wrong targets: the baker (already!), the traveller, the herb-wife, the new family.
- Correct-source identification (water tile) is a knowledge unlock — a little Snow-moment for the player.
**Educational Value:** Public health history is a history of blaming the wrong thing.
**Fun Factor:** Witch-hunt risk during epidemics; the player's info edge is actionable.
**Implementation:** extend `ergot()`'s blame pattern to outbreak kinds; `Problem::Blamed` exists.

## Naming Table — existing strings → Victorian names

| Current in code | Victorian name | Why |
|---|---|---|
| "flu" (traveller, FLU_DAY) | "la grippe" (display: "the influenza") | Russian-flu era name; traveller-borne pandemic fits the 1889 model |
| "the fever" (FeverYear scenario) | "scarlatina" | the era's defining "fever year" disease; contagious, bedridden start fits |
| "saint anthony's fire" | keep (period folk name Victorians still used) | ergotism was past its peak but the name is exactly right for rye+bread blame |
| "bad belly" (death-cap) | "the gripes" | period colloquialism for abdominal poisoning/colic |
| test names "flu"/"plague" | "la grippe" / "the sweating sickness" or keep generic | tests can stay generic |
| generic recovered/ill log strings | derive display name from `DiseaseKind` | "Agnes has the scarlatina", "recovered from the influenza" |

## Next Steps

- **Compare against implementation** (immediate task): rename strings per table above; add `DiseaseKind` with the roster's fields; wire `Stage::Latent` as real incubation.
- Backlog candidates: waterborne-via-effluent spread; seasonal outbreak table; trade-exposure occupational diseases; anti-vaccination-league organizing event (Vaccination Act as the oppression, Leicester Method as the counter-play); Snow-moment water-source investigation goal.
- Open questions: does the game's village map effluent to a *water source* tile, or just ground? Needed for Mechanic 3. What is a "day" in ticks? (365-tick year is implied by `tick % 365`; incubations must scale accordingly — a 3-day cholera incubation = 3 ticks if 1 tick = 1 day.)

## Sources Cited
Mooney (SHM 2007); GRO death-registration data via PMC7404362 & UKDA SN 5705/9034; Snow 1849 (primary); Brody/Vinten-Johansen Lancet 2000 & 2005; Science Museum cholera history; London Museum Bazalgette; BBC Great Stink; Hardy *Epidemic Streets* (OUP 1993); Hardy "Typhus in the Victorian City" (Medical History); Continuity & Change phthisis paper 2015; PMC6204963 TB trends; "The Filth Disease" (Boydell); Lancet "Beyond Bazalgette"; Porter & Porter anti-vaccination (Medical History); Leicester Method paper; Lancet 1890 anti-vax commission (primary); Gale ADC 1945; Hardy "Rickets and the Rest" (SHM 1992); CampOP "shitscapes"; Dobson "Fenland ague" (Medical History); USURJ phossy jaw; OEM scrotal-cancer review; ILO C18 history PMC3923370; Valtierra PNAS 2010; Farr's nosology 1842 (primary, via johnsnow.matrix.msu.edu + one-name.org table); PubMed 20027740 terminology methods.
