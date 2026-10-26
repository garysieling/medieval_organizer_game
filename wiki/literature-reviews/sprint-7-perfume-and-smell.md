# Lit Review - Sprint 7: Perfume Chain + Sensory History of Smell

**Date:** 2026-10-25
**Domain:** History of Technology / Sensory History / Glassmaking
**Backlog items:** Perfume, Smells (shepherds smelly, perfume helps), glassblowing chain
**Sources:** Encyclopaedia Iranica "ʿAṭr" (perfume), Wikipedia "Hungary water", Wikipedia "Enfleurage", J. Archaeological Science Reports on Murano medieval glass (2021), Kiser/Palmer *The Sense of Smell in the Middle Ages* (Routledge), "London Smellwalk around 1450" (Oxford Handbook of Later Medieval Archaeology)

## Sources

### 1. ʿAṭr - Perfume in the Islamic/Persian World (Encyclopaedia Iranica)
- Perfume (*ʿaṭr / ṭīb*) as both a substance and a quality of people/plants
- Persian and Arab world perfected **steam distillation of rose water** (Avicenna is traditionally credited with refining it); spice and aromatic trade routes from the Indies
- **Relevance:** HIGH - perfume knowledge arrives as foreign/traded tech, which gives a reason for the translator/traveler role

### 2. Hungary Water (Wikipedia, citing historical recipes)
- One of the **first alcohol-based perfumes in Europe** (~14th c., legend ties it to a Queen of Hungary)
- Recipe: **fresh rosemary (+ thyme) distilled with brandy**; later lavender, mint, sage, marjoram, orange blossom, lemon
- Sold as both **perfume and medicine** (rubbed on, even drunk)
- **Relevance:** CRITICAL - exactly the backlog chain: garden herbs + alcohol (+ glass to store it)

### 3. Enfleurage (Wikipedia)
- **Hot enfleurage** (stir flowers into warm fat, strain, repeat) is the **oldest known** way to capture scent
- **Cold enfleurage** (petals on fat-smeared glass frames for 1-3 days, replaced repeatedly) is later, developed in Grasse (17th-18th c.)
- Fat → "pomade"; washing pomade in alcohol pulls scent into the alcohol → "absolute"
- **Relevance:** HIGH - a no-still, low-tech route (fat) vs a high-tech route (alcohol + still) gives tiered perfume quality

### 4. Murano Medieval Glass (J. Archaeological Science Reports 2021)
- Venice/Murano was **Europe's leading glassmaking center**; glassmakers were moved to Murano island from the late 13th c.
- Consistent soda-ash recipes, i.e. **guarded, standardized know-how**
- (General history: Venetian glassmakers were famously restricted from leaving, so the trade secret was a state asset)
- **Relevance:** HIGH - glass is the bottleneck for storing perfume, and a model of guild/state-guarded secrets

### 5. The Sense of Smell in the Middle Ages + London Smellwalk c. 1450
- Smell was **ubiquitous and meaningful**: incense, spices, cooking, refuse, animals, tanneries
- Scholastics built a theory of smell; **miasma** (bad air causes disease) shaped city rules
- Smell carried **moral and social meaning**: sanctity had an "odor of sanctity"; stench marked sin, poverty, low trades
- Cities zoned stinking trades (tanners, butchers, fishmongers) and regulated refuse; breathing the city was "a communal affair"
- Pomanders and sweet herbs were carried against plague air
- **Relevance:** CRITICAL - smell as a social-status signal and a (wrong but believed) health model

## Mechanics

### Perfume production tiers
| Tier | Method | Needs | Output |
|---|---|---|---|
| 1 | Strewing herbs, sachets, pomanders | garden herbs | weak, short-lived |
| 2 | Hot enfleurage | flowers + fat | pomade (rubbed on) |
| 3 | Distilled waters (rose water) | flowers + still | floral water |
| 4 | Alcohol perfume (Hungary water) | herbs + aqua vitae + still + **glass bottle** | strong, lasting, tradeable |

### Smell model
- Each entity/tile has an **odor vector** (e.g. animal, smoke, rot, floral, incense) with intensity and decay
- Sources: shepherds/goats (animal), tanneries (rot), kitchens (food), churches (incense), gardens (floral)
- **Social effect:** others' first impressions shift with odor; class expectations differ (a noble court penalizes "animal" more than a village does)
- **Belief effect (miasma):** characters *believe* stench causes illness, so they avoid smelly areas and zone trades. Actual disease uses `sim/epidemic.rs`; the gap between belief and reality is itself content
- Perfume masks/overrides odor for a duration and becomes a status good

## Design Proposals

1. **Perfumer vocation** (on the ancient-jobs list) consumes garden flowers, aqua vitae, and glass. It is a good capstone for the garden, distillery, and glassblower chains
2. **Glassblower as a guarded trade:** the town can protect the secret (glassblowers can't emigrate) or lose it to a rival when someone defects
3. **Odor layer** on the map (like weather/climate layers): diffuses and decays, drives NPC pathing preferences and zoning decisions
4. **Smell-as-status:** perfume, incense, and bathing raise standing in courts. Shepherds have a smell penalty in town but fit in on the hills
5. **Medicine/perfume overlap:** Hungary water is both, so it plugs into the herbology/apothecary tree from [Sprint 4](sprint-4-tea-herbology.md)

## Rating
**High** - perfume is a satisfying 3-chain capstone (garden + distillery + glass), and smell is an unusual, cheap-to-simulate social layer with real historical grounding (miasma, odor of sanctity, zoned trades).
