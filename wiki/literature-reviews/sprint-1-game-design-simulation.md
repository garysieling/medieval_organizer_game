# Literature Review - Sprint 1: Game Design & Simulation

**Date:** 2026-10-25
**Domains:** Game Design Research + Simulation Theory
**Agents:** Credibility Analyst, Creative Filter, Emergent Effects Researcher
**Sources Reviewed:** 5 academic papers

## Sources Reviewed

### 1. "Simulation as a game design tool" (2009)
- **Source:** ACM ACE Conference
- **URL:** https://doi.org/10.1145/1690388.1690427
- **Credibility:** High (peer-reviewed, 11 citations)
- **Key Finding:** Use gameplay simulations at the *logical event level* early in development. Abstract away details to reveal long-term dynamics problems before prototyping.
- **Relevance:** HIGH - Directly supports our approach of simulating game systems before building UI. Validates our "backend API playtester" concept.

### 2. "Simulating Mechanics to Study Emergence in Games" (2011)
- **Source:** AAAI AIIDE Conference, Joris Dormans
- **URL:** https://doi.org/10.1609/aiide.v7i3.12477
- **Credibility:** High (peer-reviewed, 34 citations, established researcher)
- **Key Finding:** The Machinations framework models games as resource flow diagrams. Can simulate and *balance* games before they're built. Demonstrated on SimWar (Will Wright design, never built).
- **Relevance:** HIGH - Machinations diagrams could model our economic systems. Resource flows = mechanics. A tool for balancing our medieval economy before implementation.
- **Action:** Add backlog item - "Machinations-style resource flow modeling"

### 3. "Games and Agents: Designing Intelligent Gameplay" (2009)
- **Source:** International Journal of Computer Games Technology (Utrecht University + TNO)
- **Credibility:** High (peer-reviewed journal)
- **Key Finding:** Multiagent platforms assume agent autonomy; game engines assume central control. Integrating intelligent NPCs requires *design methodology* changes, not just technical ones. Three problem areas: synchronization, information representation, communication.
- **Relevance:** HIGH - Directly addresses our agent-driven NPC architecture. Confirms autonomy vs control tension we identified in "indirect control" mechanics.

### 4. "Exploring Gameplay With AI Agents" (2018)
- **Source:** AAAI AIIDE Conference (EA - Sims Mobile team)
- **URL:** https://doi.org/10.1609/aiide.v14i1.13034
- **Credibility:** High (peer-reviewed, 18 citations, real industry application)
- **Key Finding:** EA built AI agents that recreate bare-bone game mechanics (not the client) to playtest. Thousands of sims in minutes vs days of human testing. Found imbalances, inconsequential rewards, evaluated strategic choices. Findings changed Sims Mobile design.
- **Relevance:** CRITICAL - This is literally what we want our Full Game Playtester agent to do. Validates the whole architecture. Key insight: *recreate mechanics as separate system*, don't drive the client.

### 5. "An Emergent Approach to Game Design" (2006)
- **Source:** PhD thesis, Penelope Sweetser, University of Queensland
- **Credibility:** Medium-High (PhD thesis, well-cited in emergence literature)
- **Key Finding:** EmerGEnT system - cellular automata for emergent game worlds. Player enjoyment improves with flexible worlds giving more freedom/control. Emergent approach creates open worlds vs scripted shallow gameplay.
- **Relevance:** HIGH - Foundational for our emergent world vision. Cellular automata candidate for world simulation layer.

## Key Themes Across Sources

1. **Simulate before building** - Multiple sources (1, 2, 4) independently validate simulation-first design. Our architecture is aligned with research.

2. **Emergence from simple rules** - Sources 2, 5 confirm emergent gameplay requires simple rules + complex interactions, not scripted complexity.

3. **Autonomy vs control tension** - Source 3 names the exact tension we face with indirect control mechanics (Majesty/RimWorld style).

4. **AI playtesting works** - Source 4 proves AI agents running mechanic-level sims find real balance issues. This is our Full Game Playtester.

5. **Resource flow modeling** - Machinations (source 2) offers a concrete framework for designing our economic systems.

## Actions Taken

### Backlog Items Added
- Machinations-style resource flow modeling for game economy (NEW)
- Cellular automata for world simulation layer (NEW - from Sweetser thesis)

### Design Proposals
- **Proposal:** Adopt "mechanics-as-separate-system" playtesting (EA/Sims approach). Build game mechanics as pure API layer that playtesters can drive without UI. Already aligned with our architecture - validate by keeping game logic decoupled from rendering.

### Wiki Entries
- This document (literature review summary)

## Credibility Assessment

- 4/5 sources are peer-reviewed conference/journal papers
- 1/5 is a PhD thesis (still rigorous, well-cited)
- All highly relevant to our project direction
- No red flags; findings reinforce our architecture

## Next Steps

1. Deep dive into Machinations framework (Dormans' follow-up papers exist)
2. Read Sweetser's EmerGEnT system chapters on cellular automata for game worlds
3. Implement "mechanics as separate system" validation in Sprint 2
4. Continue domain rotation: Economics + Psychology/Sociology next sprint

## Sprint Productivity Rating

**High** - 5 sources reviewed, 3 directly validate core architecture decisions, 2 new concrete mechanics identified (Machinations, cellular automata). First lit review was fruitful.
