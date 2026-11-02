# Lit Review - Sprint 7: Mushrooms - Psychoactive, Poisonous, and the Lookalike Problem

**Date:** 2026-10-25
**Domain:** Ethnomycology / Toxicology / Medieval Medicine
**Backlog items:** Magic mushrooms (incl. poisonous lookalikes); relates to Caves, Medicinal plants, Pandemics/illness
**Sources:** Akers et al. "On the Origin of the Genus Psilocybe and Its Potential Ritual Use in Ancient Africa and Europe" (Economic Botany 2011); "Historical overview of psychoactive mushrooms" (Inflammation and Regeneration 2009); Horowitz & Moss "Amatoxin Mushroom Toxicity" (StatPearls, 2023); "Amatoxin-Containing Mushroom Poisonings" (Wilderness & Environmental Medicine 2018); Cervellin et al. "St. Anthony's fire revisited" (Acta Biomed 2021); Wikipedia *Psilocybe semilanceata*, *Amanita muscaria*

## Sources

### 1. Psilocybe Origins and Ritual Use (Economic Botany 2011) + Historical Overview (2009)
- Psilocybin mushrooms are old and widespread; their role in prehistoric European/African art is **debated** (neuropsychological model of rock art)
- Wasson's (contested) theory: Vedic **soma** was *Amanita muscaria*
- **Relevance:** MEDIUM - the uncertainty itself is useful; mushroom cults can be rumor, legend, or real in-world

### 2. *Psilocybe semilanceata* - Liberty Cap (Wikipedia)
- Europe's most widespread psilocybin mushroom: small (≤2.5 cm) conical cap with a **nipple-like tip**, grows in **wet grassland** (feeds on decaying grass roots, not dung directly), dark purple-brown spores
- **Relevance:** HIGH - the native "magic mushroom" for a medieval European setting; found in pastures, so shepherds would know it

### 3. *Amanita muscaria* - Fly Agaric (Wikipedia)
- The iconic red cap with white warts; **mycorrhizal** with birch/pine (grows only near certain trees)
- Name possibly from medieval use to **kill flies** (and the belief that flies in the head cause madness)
- Psychoactive via **muscimol and ibotenic acid** (not psilocybin); poisoning is common but **deaths are rare**
- **Relevance:** HIGH - visually distinct; dangerous-but-rarely-lethal is a good gameplay middle tier

### 4. Amatoxin Poisoning (StatPearls 2023; WEM 2018)
- **~95% of mushroom-poisoning deaths** worldwide come from amatoxin species: *Amanita* (death cap *A. phalloides*, destroying angel *A. virosa*), *Lepiota*, *Galerina*
- Classic course: symptoms are **delayed 6+ hours**, then GI phase, then **false recovery**, then liver failure days later
- **Galerina marginata** is a little brown mushroom that looks like edible/psychoactive LBMs, the textbook lookalike danger
- **Relevance:** CRITICAL - delayed onset and false recovery make a great mystery/tension mechanic (you don't know you're poisoned)

### 5. St. Anthony's Fire / Ergotism (Acta Biomed 2021)
- "St. Anthony's fire" named at least three diseases; one is **ergotism**, poisoning from *Claviceps purpurea* fungus on **rye**
- Convulsive form (fits, hallucinations) and gangrenous form (burning limbs, limb loss); **mass outbreaks** from contaminated grain in wet years
- The Order of St. Anthony (Antonines) ran hospitals for victims
- **Relevance:** CRITICAL - a fungus that hits whole villages via the food supply, read in-world as curse, witchcraft, or divine punishment

## Mechanics

### Mushroom families
| Group | Example | Effect | Lethality | Lookalike trap |
|---|---|---|---|---|
| Edible | chanterelle, field mushroom | food | - | yes (jack-o'-lantern, *Amanita* buttons) |
| Psilocybin | liberty cap | visions, insight, fear | very low | **Galerina** (deadly LBM) |
| Muscimol | fly agaric | delirium, sedation | low | - (distinct) |
| Amatoxin | death cap, destroying angel | none at first, then liver failure | **high** | edible *Amanita*, field mushrooms |
| Ergot | rye ergot | convulsions/gangrene | medium, **mass** | in the bread |

### Identification
- Each mushroom has observable traits (cap shape, gill color, ring/volva, spore print color, habitat, host tree)
- **Forager skill** sets how many traits a character can read; low skill confuses species within a lookalike pair
- Knowledge spreads socially (`sim/gossip.rs`) and can be **wrong** (folk rules like "poisonous ones tarnish silver" are false), which makes knowledge itself a gameplay object

### Effects
- Psilocybin: vision/insight event (ties to `sim/magic.rs` and the [Novel Magical Powers doc](../software-design/novel-magical-powers.md)); chance of a "bad trip" fear state
- Amatoxin: hidden poisoned state with **delayed onset + false recovery** (plug into `sim/illness.rs`)
- Ergot: wet harvest + rye → village-wide outbreak → religious/witch-hunt interpretations (`sim/religion.rs`, magic stance axiom "miracles-yes-sorcery-no")

## Design Proposals

1. **Forager/herbwife vocation:** mushroom ID is skill + local knowledge; an elder forager is valuable and can teach (apprenticeship)
2. **Lookalike pairs** as a core teaching mechanic: the game teaches real ID traits (educational goal from the Literature Survey Agent rules)
3. **Hidden poison state** with delayed symptoms creates murder-mystery and accident storylines; antidote/care via apothecary
4. **Ergot outbreak as world event:** triggered by climate (wet year) + rye-heavy diet; who gets blamed is an emergent social story
5. **Mushroom visions** as a magic source: shamanic/visionary path, contested by church authority (Galileo-style "dangerous knowledge" angle)
6. **Caves and habitat:** cave/underground mushrooms as a monochrome-zone resource (ties to the Caves item)

## Safety note
In-game ID content must never be presented as real-world foraging advice. Add a short disclaimer wherever real traits are shown. (Done: `game/mycology.rs::FORAGING_DISCLAIMER`, PLAY.md foraging note.)

## Implementation status (2026-10-31)

| Proposal | Where it landed |
|---|---|
| Mushroom families | `game/mycology.rs::MushKind` - 8 kinds across the five families, spec table carries effect/food/lookalike/trait lines |
| Lookalike pairs | `MushSpec.lookalike` - chanterelle↔jack-o'-lantern, field↔death-cap, liberty cap↔galerina (the LBM trap) |
| Hidden poison, delayed onset | `AilPhase` queue in `session.rs`: belly a day on → false mend → liver, the apothecary's only window |
| Identification by skill | `shroom_take`: `read_chance` scaled by craft skill; a true read names the species with a trait line |
| Knowledge as gameplay | `shroom_lore`: poisoned foragers learn the patch and skip it; folk rules ("the silver stayed bright") can confidently mis-clear a death-cap |
| Elder forager teaches | `teach_shrooms`: the herb-wife passes deadly-patch lore to neighbours standing near |
| Mushroom visions | liberty cap / under-cap: Joy satisfied, 20% a "fearful turn"; the priest preaches against the little caps |
| Fly agaric middle tier | `MushEffect::Delirium` - "the red cap's dream", wild not lethal; grows under birch |
| Ergot outbreak | wet-harvest-scaled trigger; convulsive + burning (gangrenous) forms; Antonine hospice care if a church stands; emergent blame (baker → herb-wife → scapegoat) + priest's "judgment" reading |
| Caves/habitat | under-cap patch at the cave; liberty caps in the wet grassland by the fold |
| MushroomFlush event | real implementation: every patch flushes + bonus caps, picked through at the stay's end |

## Rating
**Critical** - mushrooms give knowledge-as-gameplay (lookalikes), hidden-state drama (delayed poisoning), a mass-event generator (ergot), and a grounded magic/vision source.
