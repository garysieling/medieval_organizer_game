# Literature Review: Anti-Colonialist / Liberation Literature

**Sprint:** 4 — Thematic Literature
**Backlog item:** "Anti-Colonialist/Liberation Literature Survey" (High)
**Date:** 2026-10-25

## Description

Survey of liberation/anti-colonial thought — the theoretical grounding for the game's political/allegory layer (`medieval-allegory-power-dynamics.md`). These works describe how domination works *structurally* and how people resist it — the intellectual core of the "organizers vs extractive elites" game.

## Core works

### Frantz Fanon — *The Wretched of the Earth* (1961)
- Colonialism as total system: the colonized are made objects; liberation requires *reclaiming subjectivity* through action
- Violence as both colonial tool and decolonial response — not endorsement, but diagnosis: colonialism IS violence; it can't be petitioned away
- **Game relevance:** the psychology of domination and its psychic cost; revolt as a *psychological necessity*, not just a political choice — colonized people's inner experience of the system, which our psych sim can model

### Paulo Freire — *Pedagogy of the Oppressed* (1970)
- "Banking education" vs problem-posing education — the oppressed are treated as empty vessels; liberation requires them to *name their world*
- Conscientization: awakening through dialogue and praxis — oppressed people become subjects by critically reading their reality
- **Game relevance:** organizing as *consciousness-raising*; a community's critical awareness is a mechanic (literacy, meetings, `community.rs` house meetings ARE Freire's praxis); the priest/teacher as either domesticator or liberator

### Edward Said — *Orientalism* (1978)
- The "Orient" as a Western construction — knowledge production is power; representing a people is a *political act*
- The colonized as objects of the colonizer's scholarship, map, census
- **Game relevance:** who *describes* a village matters — the lord's scribe's account vs the villager's reality; maps, censuses, and chronicles as instruments of power (fits `civ-style-text-network.md` information asymmetry)

### James C. Scott — *The Moral Economy of the Peasant* (1976), *Weapons of the Weak* (1985), *Seeing Like a State* (1998)
- **Moral economy:** peasants live by a subsistence ethic + reciprocity norms — extraction is judged *illegitimate* when it violates the safety-first norm; revolt triggers when the lord breaks the subsistence floor
- **Weapons of the weak:** everyday resistance — foot-dragging, pilfering, gossip, false compliance — not revolts but *erosion*; the quiet way peasants actually fight
- **Seeing like a state:** legibility projects — the state simplifies reality (cadastral maps, surnames, standard measures) to control it; "simplification" destroys local knowledge and produces its own failures
- **Game relevance:** THE most directly applicable — the moral economy IS the villagers' legitimacy model (`legal-framework-contracts.md`); everyday resistance is already how agents resist in a society sim; legibility is what the ruler's census does to the villagers

### Silvia Federici — *Caliban and the Witch* (2004)
- Primitive accumulation required not just land enclosure but the *destruction of women's autonomy* — the witch hunts as a project to discipline labor and reproduction
- The commons and the social reproduction it supported as the target of enclosure
- **Game relevance:** enclosure mechanics (`legal-framework-contracts.md`); the witch figure as a real social actor (`novel-magical-powers.md`, `herbology-apothecary-tree.md`); reproduction/labor as economic categories — women's unpaid work as the hidden engine

### CLR James — *The Black Jacobins* (1938)
- The Haitian Revolution as slave revolt meeting French Revolution ideas — the enslaved as historical agents, not victims
- Revolution as both political event and mass consciousness transformation
- **Game relevance:** revolt mechanics — how organized uprising happens: consciousness + organization + opportunity (`community.rs`, `fundamentalism.rs`)

### Chinua Achebe — *Things Fall Apart* (1958)
- The interior world of a colonized society — Okonkwo's story makes colonialism legible through *someone's life*
- **Game relevance:** the interior view — a society sim is exactly this: showing a world from inside the people living it; the tragedy is structural not villainous

### Albert Memmi — *The Colonizer and the Colonized* (1957)
- The colonizer benefits from the system regardless of intention; the colonized is dehumanized *and* constructed as "the native" — a role, not a person
- **Game relevance:** role-vs-person modeling — villagers under extraction are *roles to the ruler*, persons to each other; the dehumanizing lens is mechanical (how the lord's census "sees" them)

## Synthesis for game2

| Concept | Where it lands |
|---|---|
| Moral economy / subsistence floor | villager legitimacy threshold — extraction above the floor triggers resistance (`legal-framework-contracts.md`) |
| Weapons of the weak | foot-dragging, pilfering, gossip as resistance behaviors — the default resistance mode before open revolt |
| Legibility / seeing like a state | the ruler's census/map simplifying villagers — power through description (`civ-style-text-network.md`) |
| Conscientization | house meetings / literacy / critical awareness as a mechanic (`community.rs`, `printmaking-tech-tree.md`) |
| Witch hunts / enclosure of reproduction | `herbology-apothecary-tree.md`, `novel-magical-powers.md`, the hedgewitch as social threat |
| Revolt mechanics | `sim/community.rs`, `sim/fundamentalism.rs` — organized uprising as emergence |

## Rating

The intellectual spine of the game's political layer — Scott especially is nearly a mechanic spec.
