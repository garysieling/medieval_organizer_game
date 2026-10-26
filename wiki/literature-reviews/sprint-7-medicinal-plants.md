# Lit Review - Sprint 7: Medicinal Plants and Physic Gardens

**Date:** 2026-10-25
**Domain:** History of Medicine / Ethnobotany / Pharmacology
**Backlog items:** Medicinal plants (growable in gardens); House and community gardens. Extends the [Sprint 4 tea + herbology review](sprint-4-tea-herbology.md)
**Sources:** Boseva & Bosseva "Edible and medicinal plants in the cloister gardens of West Europe (800s-900s AD)" (Phytologia Balcanica); Plan of St. Gall *herbularius* (Archaeological Journal vol. 5); Capitulare de villis plant list (biozac.de); MDPI *Religions* "Saint Hildegard's Vegetal Psycho-Physio-Theology" (2018); "Hildegard's Medicine: A Systematic Science of Medieval Europe" (PRISM); Wikipedia + HerbalGram "Doctrine of Signatures"; Stone (1763) willow bark commentary (Phil. Trans. R. Soc. B, PMC4360122); Harrison et al. "A 1,000-Year-Old Antimicrobial Remedy with Antistaphylococcal Activity" (mBio 2015); "Herbal diuretics revisited: from 'wise women' to William Withering" (PubMed 12097727)

## Sources

### 1. Carolingian Garden Lists (Capitulare de villis c. 771-800; Plan of St. Gall; Walahfrid Strabo *De cultura hortorum*)
- Charlemagne's estate decree **ordered a fixed list of plants** (roughly 90 herbs, vegetables and trees) to be grown on royal estates: lily, rose, sage, rue, fenugreek, pennyroyal, mint, rosemary...
- The **Plan of St. Gall** puts a **physic garden (*herbularius*) with 16 named beds** right behind the physicians' house, next to the infirmary
- Separate kitchen garden and orchard-cemetery, so **medicine, food, and the dead are spatially planned**
- **Relevance:** CRITICAL - a real, finite, period plant list to seed the garden system; policy can mandate what gets planted

### 2. Hildegard of Bingen, *Physica* (12th c.)
- Abbess, systematized hundreds of plants, animals, stones
- Every plant is **hot or cold** (plus moist/dry in the humoral frame); "the heat of herbs signifies the spirit, the cold the body"
- **Viriditas** ("greenness"): a life force in plants and people, part theology, part medicine
- A woman writing medicine with wide scope of practice, which later centuries closed off
- **Relevance:** HIGH - the humoral hot/cold system is a simple, period-correct way to model "matching remedy to illness"

### 3. Doctrine of Signatures
- Belief that a plant's **shape/color shows what it treats** (walnut ≈ brain, lungwort's spotted leaves ≈ lungs, eyebright ≈ eyes)
- Roots in Dioscorides/Pliny; developed by **Paracelsus**; named by **Jakob Böhme**; popularized by Culpeper (17th c.)
- Mostly wrong, but it **shaped what people tried**, so it was sometimes accidentally right
- **Relevance:** HIGH - a belief system that guides experimentation; the gap between belief and efficacy is gameplay

### 4. Remedies That Actually Work
- **Willow bark** for fevers and pain (anecdotal since Hippocrates; Rev. Edmund Stone's 1763 report to the Royal Society) → salicin → aspirin
- **Bald's Leechbook eyesalve** (garlic, leek/onion, wine, ox bile, brass vessel, 9 days): a 2015 study reproduced it and found it **killed MRSA** in lab models
- **Foxglove** for dropsy, taken from folk "wise women" practice by William Withering (1785) → digitalis; **narrow gap between dose and poison**
- **Relevance:** CRITICAL - some folk remedies are real pharmacology, and preparation details matter (the vessel, the steeping time)

## Mechanics

### Plant record
```
plant { name, humor: hot|cold, moist|dry, signature: body_part?, true_effect: {..}, dose_window: (min,max), habitat, season, garden_beds: n }
```
- **True effect** is hidden; characters see the **believed effect** (humoral rule + signature + folk knowledge)
- **Dose window:** foxglove-style plants heal inside the window and poison outside it (ties to the hidden-poison state in the [mushrooms review](sprint-7-mushrooms.md))
- **Preparation** multiplies effect: infusion, decoction, poultice, salve, tincture (needs aqua vitae, see [fermentation review](sprint-7-fermentation-trades.md))

### Knowledge
- Knowledge holders: monks (written herbals), wise women (oral), apothecaries (guild), physicians (humoral theory)
- Experimentation: trying a remedy reveals a noisy signal about its true effect; good observers converge on truth, bad theory (signatures) biases trials
- **Herbals as items:** writing a herbal locks knowledge in and makes it shareable (pairs with the printmaking review)

## Design Proposals

1. **Physic garden building** with N beds; seed with the Capitulare/St. Gall list. Rulers or abbots can **mandate plant lists** (a policy lever)
2. **Hidden true vs believed effect** for every remedy; the herbology tree becomes a discovery game, not a lookup table
3. **Humoral matching:** illnesses get hot/cold tags; matching gives a placebo/belief bonus, the true effect is separate, so both matter
4. **Wise-woman vs physician tension:** folk knowledge often more accurate, but low status; can be suppressed (feeds witch-hunt dynamics from the religion system)
5. **Community garden** as a shared commons: anyone can tend and harvest; the tragedy-of-commons vs mutual-aid balance gives organizers a project
6. **Breakthrough events:** a character connecting a folk remedy to results (Stone, Withering) is a mini Galileo moment: reputation gain, or a backlash from the physicians' guild

## Rating
**Critical** - turns gardens into a knowledge-discovery system grounded in real period plant lists, real belief systems, and real pharmacology.
