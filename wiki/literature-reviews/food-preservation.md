# Medieval Food Preservation - Salt, Smoke, and the Hunger Gap

**Research Domain:** Food history, medieval economy, social history
**Survey Date:** 2026-11-11
**Researcher:** Ticket #951 deep dive
**Status:** COMPLETE
**Pairs with:** `wiki/software-design/preservation-salt.md` (the mechanic proposal)

---

## Executive Summary

Preservation was not a technique among others in the medieval village — it was
the difference between eating in March and not eating. Three findings matter
most for the game:

1. **Preservation was seasonal labor, not a recipe.** The autumn slaughter
   (Martinmas, November 11) killed every beast the village couldn't feed
   through winter, and the salting that followed was communal work — the
   year's largest coordinated household effort after the harvest itself.
2. **Salt was the choke point.** Every preservation chain that mattered —
   bacon, salt pork, salt beef, stockfish-adjacent cures, pickled anything —
   ran on salt, and salt was imported, expensive, and *taxed*. The French
   gabelle made salt a cause of revolt for four centuries.
3. **The Church's calendar created the market.** Fish days (Lent plus ~120
   fast days a year) made preserved fish one of the largest trade goods in
   Europe — the Hanseatic League effectively ran on herring and stockfish.

The existing sim already treats preservation as fragments (spoilage, cheese
aging, ale souring, a smokehouse that is flavor only). The literature says
the connective tissue is **salt as a scarce, contested input** and **the
autumn slaughter as a village-scale seasonal event**.

---

## 1. Why Preservation Mattered

**Rating:** A (consensus across all sources)
**Type:** Historical synthesis
**Relevance:** Foundational — this is the stakes of the mechanic

### The Hunger Gap

The medieval food year had two hard walls:

- **The slaughter wall (November):** Fodder was the binding constraint on
  herd size. A village could not winter more beasts than its hay stores
  carried. On Martinmas (Nov 11) the surplus livestock — most pigs, some
  cattle — was slaughtered in a few weeks. Everything not eaten fresh
  had to be salted, smoked, or dried *immediately*. There was no
  refrigeration: preservation was the only way the November kill fed
  March mouths.
- **The hunger gap (April–June):** Winter stores ran out weeks before the
  first new crops. Chroniclers mark spring as the starving season — the
  gap between the last sound stored food and the first green. Villages
  with better-preserved stores felt it as lean meals; villages without
  felt it as mortality.

### Key Numbers (for tuning, not realism)

- A salted pig gave roughly 60–70% of its live weight as storable meat.
- Salt beef/pork kept 3–6 months barrelled in brine; hard-smoked bacon a
  year; stockfish effectively indefinitely.
- Grain, well dried and kept from vermin, was the *only* food that
  routinely lasted a full year — which is why the vermin/cat mechanic
  already in `spoilage_tick` is historically the right tension.
- Medieval household salt use ran ~10–20 lb per person-year when
  preservation was counted — a real recurring expense, not a garnish.

### Game Application

Spoilage exists (`economy.rs::spoilage_tick`) but has no counter-move:
food rots on a schedule and nothing the village does extends it. The
literature says the counter-move is the core loop — **convert perishables
into keepsakes once a year, and that conversion costs a trade good.**

---

## 2. Salt - The Strategic Good

**Rating:** A (Braudel; Kurlansky's *Salt*; gabelle historiography)
**Type:** Economic/institutional history
**Relevance:** The proposed mechanic's spine

### Production and Trade

Medieval salt came from three sources, each a different economy:

- **Evaporated sea salt** — the Bay of Bourgneuf (Brittany/Normandy coast)
  and the Venetian lagoon ran solar evaporation at scale. Bulk, cheap
  where it was landed, and the backbone of the herring fleets.
- **Boiled brine (salterns)** — inland brine springs (Droitwich,
  Lüneburg, the Salzkammergut) boiled brine to crystals over enormous
  wood fires. Lüneburg's salterns were the Hanseatic League's second
  pillar after stockfish; the town's wealth was literally boiled out
  of the ground.
- **Rock salt mining** — Wieliczka (Poland), Cardona (Spain). Deep
  mining, concentrated wealth, royal monopolies almost from the start.

Salt moved along named routes — the *Salzstrassen* of Germany, the
*Via Salaria* that pre-dated Rome. It was bulk cargo: boats, then
pack-trains, then pedlars with salt in the pack's bottom.

### The Gabelle - Salt as a Grievance Machine

France's salt tax (the *gabelle*, c. 1341–1946) is the single best
documented example of a preserved-food input becoming an organizing
grievance:

- The crown declared salt a royal monopoly. In the *pays de grande
  gabelle* every household was **obliged to buy a minimum quantity of
  salt per person per year** at the royal price — the *sel du devoir*.
- Smuggling (the *faux-saunage*) was endemic and savagely punished —
  the galleys for men, and famously the condemned went "to the salt"
  themselves.
- Resistance to the gabelle runs through four centuries of revolts;
  it was still being cited in 1789 cahiers de doléances.

The institutional lesson: **a necessity good that is imported, priced
by a monopoly, and consumed by every household is a near-perfect
grievance engine.** The village's salt need is periodic and
predictable (everyone needs it at once, in November) — which makes
shortage events communal rather than private.

### Game Application

`salt` exists in `goods.rs` as a dead good (worth 3.0, no producer, no
consumer, no tags). Making it live — imported via pedlar/caravan,
consumed by the salting-bee, taxable or monopolizable by the lord —
turns an already-true historical structure into an organizing lever
with zero new systems: it's the same `Building.stock`/`Satchel`/
`trade_person` plumbing everything else uses.

---

## 3. The Preservation Methods

**Rating:** A- (monastic cellarer records; manorial accounts; Hanseatic
trade documentation)
**Type:** Technical history
**Relevance:** The conversion table for the proposal

### 3.1 Salting (dry-salt and brine)

The dominant method. Dry-salting packed meat between layers of salt in
troughs; brining submerged it in barrels. Salt pork and beef in brine
was the staple winter protein of everyone above subsistence — and
famously the ship's ration for centuries. Required roughly its weight
in salt per ~20 lb of meat. **Salt cost was the whole operation.**

### 3.2 Smoking

Usually combined with salt: a light cure, then days over a smouldering
fire (the smokehouse, or simply the smoky rafters of a chimney-less
hall). Bacon and ham were the prestige cuts; smoked herring (kippers,
red herring of Yarmouth) was the mass product. The smoke house as a
*building* appears by the high middle ages; the fisher's smokehouse
in the game (`vocation.rs:357`, flavor only today) is accurate.

### 3.3 Drying

Grains and legumes dried to keep (already implicit in the sim).
**Stockfish** — cod split and air-dried in the Norwegian cold — needed
no salt at all, kept for years, and was Bergen's entire export economy;
the Hanseatic kontor there ran the trade for 400 years. Dried peas and
beans were the village's own keep-forever staple.

### 3.4 Fermentation and pickling

Ale/beer was preservation of calories: brewing converted grain into a
sterile, storable (for weeks) drink — and small ale was the default
beverage where water was suspect. Already modeled (`drink_tick`).
Cheese was preserved milk — already modeled (`cheese_tick`, including
the aged wheel as a luxury). Pickling in vinegar or brine covered eggs,
cabbage, and whatever the garden gave in glut.

### 3.5 Cold storage

Cellars, spring-houses, and (for lords) ice houses. Temperature
mattered enormously — the same meat that spoiled in a week in summer
kept a month hung in a cold larder. `Cellar` buildings already keep
wine forever; the temperature gradient is the one part of the
literature the sim already approximates.

### Failure Modes

Preservation failed, and the failures were social events:

- **Ergotism** ("St Anthony's Fire") — ergot-infected rye stored for
  winter caused mass poisoning events, historically village-scale
  disasters interpreted as divine punishment or witchcraft.
- **Putrid meat scandals** — town records are full of market officials
  condemning spoiled meat; selling "sound" meat was a legal duty, and
  towns appointed flesh-inspectors. A village with a botched salting
  season didn't just go hungry — it ate badly and sickened, or paid
  ruinous market prices.

### Game Application

The conversion table is small and honest:

| In                    | + salt | + time/heat | = keeps months  |
|-----------------------|--------|-------------|-----------------|
| fish                  | yes    | smokehouse  | smoked-fish     |
| meat (slaughter)      | yes    | salting tub | salt-pork/bacon |
| milk                  | no     | cellar      | cheese (in)     |
| grain                 | no     | vermin-free | wheat (in)      |
| ale                   | no     | (weeks)     | vinegar (in)    |

Only the first two rows are missing. Both run on salt.

---

## 4. Preservation as Social Labor

**Rating:** B+ (village-community historiography; feast-day calendars)
**Type:** Social history
**Relevance:** The organizing angle Gary flagged — the bee, not the recipe

### The Salting-Bee Pattern

The Martinmas slaughter was household work done *together*: the kill,
the butchering, the salting ran to a shared rhythm over days. The
reasons were practical — a pig is a lot of carcass, the salt had to go
on fresh meat, and the smokehouse ran best loaded. The same communal
pattern shows up in husking-bees, barn-raisings, and harvest teams:
**concentrated seasonal labor created the year's densest social
contact.** Debts were settled, matches discussed, grievances aired —
the work bee was where the village talked to itself.

What wouldn't keep — the offal, the head, the parts that salting ruins —
was feasted on immediately. The slaughter days were simultaneously
the year's hardest work and its freshest feast.

### Preservation and Hierarchy

- Lords held **salting rights and oven rights**: the banal mill and
  bakehouse model (pay the lord's monopoly to process your own food)
  extended in places to presses and salting houses.
- Monastic cellarers managed storage logistics at institutional scale —
  the cellerer's accounts are the source for most of what we know about
  quantities.
- The lord's winter stock *was* power: hospitality, alms, and wages
  were paid out of the preserved store. A granary and a salt-beef
  barrel were a lord's bank.

### Game Application

The `festival_tick` feast machinery (May Day, Midsummer, Harvest Home,
Yule — `society.rs:957`) is exactly the shape a salting-bee needs: a
fixed calendar day, village-wide gathering, stock pulled from building
stores, social effects. Martinmas (day ~314 in the sim's 365-day year)
sits between Harvest Home (263) and Yule (355) — the slot is empty
precisely where the historical calendar put the slaughter.

---

## 5. Synthesis - What the Literature Says the Game Should Feel Like

1. **November should feel like a deadline.** The player sees beasts,
   fish, and a salt bill; the village sees a week of communal work
   that determines spring.
2. **Salt should feel like ammunition.** Whoever holds salt before the
   slaughter holds leverage — the lord, the pedlar, or the org's own
   stockpile. A salt shortage in November is a village crisis with a
   single, nameable cause.
3. **Preserved goods should read as security.** A store of salt-pork
   and smoked fish in January is visible, countable resilience — and
   the difference between a lean spring and a starving one.
4. **Failure should be social.** A bad salting season produces sour
   stores, sick villagers, and market dependence — grievances with
   someone's name attached (the lord who hoarded the salt, the
   organizer who stockpiled it early).

---

## Sources Consulted / Synthesized

- Braudel, *The Structures of Everyday Life* — preservation methods,
  quantities, the cod/herring economy. (A, foundational synthesis)
- Kurlansky, *Salt: A World History* — production sites, trade routes,
  the gabelle. (A-, trade history)
- Gabelle historiography (French salt-tax literature) — the
  monopoly/minimum-purchase mechanism and its revolts. (A-)
- Hanseatic trade documentation — Bergen stockfish, Lüneburg salterns,
  the Scanian herring fairs. (A-)
- Monastic cellarer accounts — storage logistics, seasonal rhythms. (B+)
- `agriculture-food-systems.md` (in-repo) — the existing short section
  this expands; ratings and relevance grades kept consistent.
