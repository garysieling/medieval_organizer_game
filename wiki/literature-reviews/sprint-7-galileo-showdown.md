# Lit Review - Sprint 7: The Galileo Showdown - Dangerous Discoveries vs Power

**Date:** 2026-10-25
**Domain:** History and Sociology of Science / Political Theory
**Backlog items:** Galileo-style showdown (disclosing a discovery that confronts power); relates to Astrology/sky events, Religion, Number systems
**Sources:** Stanford Encyclopedia of Philosophy "Galileo Galilei"; Finocchiaro *The Trial of Galileo* (UNC Press) and Fantoli/Blackwell editions (U. Toronto Press "The Trial of Galileo, 1612-1633"); *Church History* "Galileo's Non-Trial (1616), Pre-Trial (1632-1633), and Trial (1633)"; Barber "Resistance by Scientists to Scientific Discovery" (*Science* 134, 1961); Wikipedia "Semmelweis reflex"; *BJHS* "Why Semmelweis's doctrine was rejected"

## Sources

### 1. Galileo Affair (SEP; Finocchiaro; *Church History*)
- 1610: telescope discoveries (Jupiter's moons, Venus phases) undermine geocentrism; Galileo uses them to win **Medici patronage** (names the moons "Medicean stars")
- 1616: Copernicanism declared "formally heretical"/contrary to Scripture; Galileo **admonished** not to hold or defend it (the "non-trial")
- 1632: publishes the *Dialogue*; the geocentric speaker "Simplicio" was read as mocking **Pope Urban VIII**, a former friend and patron
- 1633: tried by the Inquisition for "vehement suspicion of heresy", **forced to abjure**, *Dialogue* banned, **house arrest for life**
- Historians stress it was **not only science vs religion**: patronage politics, a personal insult, the Counter-Reformation's need to look firm, and procedural irregularities all mattered
- **Relevance:** CRITICAL - the outcome depended on **relationships and timing**, not just the truth of the claim

### 2. Resistance by Scientists to Scientific Discovery (Barber 1961)
- Resistance comes from **inside** the establishment too, not just outsiders
- Sources: prior theory, methodological conceptions, religious ideas, **professional standing** (low-status discoverers ignored), specialization, **organizational** loyalty (schools, journals)
- **Relevance:** HIGH - the "powerful" opposing a discovery include guilds and experts whose status depends on the old idea

### 3. Semmelweis Reflex (Wikipedia; BJHS)
- Semmelweis (1840s) showed handwashing cut childbed fever deaths; doctors **rejected** it: it implied doctors were killing patients, it lacked a theory (pre-germ theory), and his communication was combative
- "Semmelweis reflex" = reflexive rejection of evidence that contradicts established norms
- **Relevance:** HIGH - **how** you disclose matters (tone, allies, explanation) as much as **what**

## Mechanics

### Discovery object
```
discovery { claim, truth: bool, evidence: 0..1, threatens: [orgs/beliefs], explanatory_theory: bool }
```

### Disclosure as a scenario
- **Threat score** for each power holder = how much the claim undermines their legitimacy, income, or doctrine (geocentrism ↔ church authority; handwashing ↔ physicians' competence)
- **Reaction** = f(threat, discoverer's standing, insult factor, political climate, discoverer's allies)
  - ignore → ridicule → admonish → suppress (ban the book) → trial → exile/house arrest/death
- **Disclosure choices** (player or NPC): private letter to a patron, coded/fictional framing (the *Dialogue* format), public pamphlet, teaching students quietly, waiting for a sympathetic ruler
- **Insult multiplier:** framing that humiliates a specific powerful person (Simplicio) spikes the reaction
- **Truth wins slowly:** suppressed true discoveries keep spreading underground through students and copies; over generations they become orthodoxy (the Church lifted the ban on heliocentric books in the 1700s-1800s)

## Design Proposals

1. **Discovery events** from astronomy (comets, eclipses, see Astrology backlog item), medicine, mathematics (number systems), each with a list of threatened orgs
2. **Disclosure strategy** as a player decision with the reaction model above. Allies (patrons, guild, students) buffer the backlash
3. **Patronage double edge:** a patron protects you until the discovery embarrasses them (Medici vs Urban VIII)
4. **Underground spread** of suppressed knowledge via the gossip/printing systems; eventual vindication as a long-term arc
5. **Insiders resist too** (Barber): physicians' guild resisting Semmelweis-style findings; a discovery can split an org into factions
6. **Abjure or stand firm** as a moral choice with consequences: survive and keep working, or martyrdom that boosts the idea's spread

## Rating
**High** - a reusable scenario template ("truth vs power") where outcomes depend on relationships, framing, and timing; it fits the organizing, religion, and knowledge systems.
