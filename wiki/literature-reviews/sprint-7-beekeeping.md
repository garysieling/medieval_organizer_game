# Lit Review - Sprint 7: Beekeeping + the Hive as a Faction Template

**Date:** 2026-10-25
**Domain:** Medieval Economic History / Entomology
**Backlog items:** Beekeeping (model the few kinds), hive social structure as a faction template
**Sources:** Sales Fava, Sapoznik & Whelan "Beekeeping in late medieval Europe" (Anales Univ. Alicante, 2021); Sapoznik "Bees in the medieval economy: wax in England c.1300-1555" (Economic History Review); Mortensen, Smith & Ellis "The Social Organization of Honey Bees" (UF/IFAS ENY-166); Wikipedia "Beehive" (skeps, log hives)

## Sources

### 1. Beekeeping in Late Medieval Europe (Sales Fava et al. 2021)
- Bees had big economic, social, and cultural weight: **wax** demand (Christian worship) and **honey**, the only widely available sweetener before sugar imports
- **Range of keepers:** peasants with a few hives through to **specialist beekeepers** producing for international trade
- **Different environments → different forms of beekeeping**
- Beekeeping **competed and conflicted with other resource uses** (forest grazing, wood cutting, lordly rights)
- Real expertise, learned by practice: track swarm size, feeding, cleanliness, predators, partial honey extraction
- **Relevance:** CRITICAL - beekeeping is a land-use conflict as well as a craft

### 2. Bees in the Medieval Economy: Wax (Sapoznik, EHR)
- **Wax candles were needed at every level of society** (light/dark imagery in Christian devotion)
- England had ~**9,000 parish churches** by the 13th c., so wax demand was enormous
- Hanseatic **Baltic forest wax** was a major export (with fur, wood, metals), but domestic production must have been much larger than imports
- Wax was used as **payment in kind** (officers paid in candles by rank) and for **sealing wax** (documents, legal power)
- Beekeeping is nearly absent from manorial accounts, so it was **hidden in small household economies**
- **Relevance:** CRITICAL - wax links bees to religion (`sim/religion.rs`), law/contracts (seals), and light

### 3. Kinds of Beekeeping (synthesis: Wikipedia "Beehive", Sales Fava et al.)
- **Forest/tree beekeeping** (Baltic, Slavic lands, German *Zeidler*): bees in hollowed living trees in the forest, marked by owners; forest-beekeeper guilds held rights in lords' forests
- **Log hives:** hollow log sections set upright or stacked near homes
- **Skeps:** coiled straw/wicker baskets (often daubed). Cheap and portable, but honey harvest usually **killed or drove off the colony**
- **Clay/pipe hives:** Mediterranean/Middle East horizontal clay cylinders, stackable walls of hives
- (Removable-frame hives are 19th c., so they are out of period and could be a late-tech unlock)

### 4. Social Organization of Honey Bees (UF/IFAS ENY-166)
- True **eusociality**: overlapping generations, cooperative brood care, reproductive division of labor
- **Queen:** the only reproductive female; her pheromones hold the colony together
- **Workers:** sterile females who change jobs with age (cleaning → nursing → building → guarding → foraging)
- **Drones:** males whose only job is mating; they **don't forage or defend** and are **expelled to die in autumn** when resources tighten
- **Swarming:** the old queen leaves with part of the colony to found a new one, which is how colonies reproduce
- **Relevance:** HIGH - matches the backlog note (matriarchal, males don't work, die in winter)

## Mechanics

### Beekeeping
| Kind | Where | Cost | Harvest | Notes |
|---|---|---|---|---|
| Forest/tree | forest tiles | rights from lord | moderate, sustainable | ownership marks, forest-use conflict |
| Log hive | near home | low | moderate | sedentary |
| Skep | anywhere | very low | high but **kills colony** | must catch swarms to restock |
| Clay pipe | warm/dry regions | medium | sustainable | stackable hive walls |

- **Outputs:** honey (food, mead, medicine), wax (candles → church demand, seals → contracts)
- **Pollination bonus** to nearby gardens/orchards (ties to the gardens and perfume flowers items)
- **Swarm events:** colonies split in spring; catching a swarm is a free hive and a classic folk-skill moment

### Hive faction template
- **Roles shift with age** (a career ladder by age, not by choice)
- One central reproductive authority whose "pheromone" (legitimacy) holds the group together; if she is lost, crisis follows until a successor emerges
- **Non-working caste tolerated in good times, expelled in hard times** (seasonal cruelty, usable as allegory)
- **Swarming = schism:** when a faction grows too big, the old leader leaves with followers to found a new settlement (compare Stark's sect cycle in the religion review)

## Design Proposals

1. **Beekeeper vocation** with the 4 hive kinds as tech/region variants; skep vs sustainable choice = short-term vs long-term tension
2. **Wax economy:** churches consume candles continuously; scarcity of wax = darkened churches, which hurts piety/morale. Seals require wax, so contracts require bees
3. **Forest rights conflict:** forest beekeepers vs woodcutters vs lord's hunting, an emergent dispute generator (ties to the legal framework doc)
4. **Swarm-split as faction growth rule:** any org above a size/strain threshold can "swarm", splitting into a daughter org with the old leader, used for guilds, sects, villages
5. **Hive faction template** for a non-human or allegorical faction (matriarchal, age-based roles, drones expelled in winter)
6. **Mead** joins the fermentation family (honey + water + time)

## Rating
**High** - small system, many hooks: religion (wax), law (seals), farming (pollination), fermentation (mead), land-use conflict (forest rights), and a reusable faction-splitting rule from swarming.
