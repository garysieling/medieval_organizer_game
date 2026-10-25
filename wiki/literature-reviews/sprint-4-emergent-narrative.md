# Lit Review - Sprint 4: Emergent Storytelling

**Date:** 2026-10-25
**Domain:** Computational Narrative / Emergent Systems
**Sources:** Ryan/Mateas/Wardrip-Fruin (UCSC), IEEE ToG, FDG, Kreminski+Mateas

## Sources

### 1. Open Design Challenges for IEN (Ryan/Mateas/Wardrip-Fruin)
- **Emergent narrative:** stories emerge bottom-up from autonomous characters in simulated storyworlds (Dwarf Fortress, The Sims)
- **4 design challenges:** modular content, compositional representational strategies, story recognition, story support
- **Key gap:** sim tuning doesn't improve the interactive experience itself
- **Relevance:** CRITICAL - directly addresses our emergent storytelling vision

### 2. Tension Space Analysis for Emergent Narrative (IEEE ToG 2020)
- **Narratological possible worlds:** which stories can emerge is a formal analysis problem
- **Expressive range analysis:** measure a system's story potential
- **Relevance:** HIGH - we could analyze our system's story space

### 3. Narrative Substrates (FDG 2020)
- **Persistence + emergence = player influence, not restriction**
- **Players value first-time/unique events, externalize to web**
- **"We Ride" game:** 2-phase probe over 1 year, reifying emergent narratives
- **Relevance:** CRITICAL - persistence is what makes emergence meaningful

### 4. Coauthorship-Centric History of IEN (Kreminski+Mateas 2021)
- **Two uses:** player participation as character vs player authorship of stories
- **Need distinction:** systems for participation vs authorship evaluated differently
- **Relevance:** HIGH - we're doing both: player participates (outsider building influence) + authors (organizing)

## The 4 Design Challenges (for us)

1. **Modular content** - story elements should be reusable/composable
2. **Compositional representational strategies** - how to represent events/state for emergence
3. **Story recognition** - the system needs to recognize when a story emerges (not just produce it)
4. **Story support** - help the player recognize/experience the emerging story

## Key Insights

- **Story emergence is bottom-up:** autonomous characters + simulation -> narratives emerge, not scripted
- **Persistence is the multiplier:** persisted changes make emergence meaningful (player influence)
- **Participation vs authorship:** we're both - player participates as outsider AND authors through organizing
- **Story recognition matters:** the system must recognize emergent stories, not just produce events

## Design Proposals

- **Modular story elements:** reusable narrative components (betrayal, alliance, discovery, triumph)
- **Event representation:** state changes produce narrative-relevant events (not just numbers)
- **Story recognition:** detect emergent patterns (villager X got sick -> treated -> recovered -> bonded)
- **Story support:** surface emergent stories to player (relationships, events, outcomes)
- **Persistence:** save/load preserves emergent stories across sessions

## Rating
**Critical** - Emergent storytelling is our core vision. These papers validate the approach and identify exactly the challenges we need to address (modularity, representation, recognition, support).
