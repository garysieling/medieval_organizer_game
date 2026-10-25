# Literature Review Scheduling System

## Overview

Regular literature reviews to find knowledge beyond what can be provided from memory. The system schedules periodic research tasks to discover new academic papers, books, articles, and resources that can inform game design, simulation mechanics, and novel systems.

## Goals

1. **Discover Novel Mechanics** - Find academic research that can be translated into game mechanics
2. **Validate Existing Designs** - Find research that supports or challenges current design decisions
3. **Inspire Innovation** - Discover cross-disciplinary ideas (e.g., QSP for non-medical applications)
4. **Fill Knowledge Gaps** - Research areas where current knowledge is insufficient
5. **Stay Current** - Keep up with new research in relevant fields

## Research Domains

### Primary Domains (High Priority)
- **Game Design Research** - Academic game design papers, game studies, player psychology
- **Simulation Theory** - Agent-based modeling, complex systems, emergent behavior
- **Economics** - Behavioral economics, microeconomics with lifespan, alternative economic models
- **Psychology/Sociology** - Attachment theory, social dynamics, group behavior, influence systems
- **Pharmacology/QSP** - Quantitative Systems Pharmacology, ADME/ADAM models

### Secondary Domains (Medium Priority)
- **History** - Medieval history, printmaking history, tea history, textile history
- **Theology/Philosophy** - Dante's cosmology, religious studies, ethical frameworks
- **Political Science** - Community organizing, power dynamics, anti-colonial theory
- **Biology/Medicine** - Illness modeling, aging, lifecycle simulation
- **Art/Design** - Isometric art, pixel art techniques, visual design principles

### Tertiary Domains (Low Priority - Exploratory)
- **Emergent Systems** - Fractals, cellular automata, complexity theory
- **Narrative Theory** - Emergent storytelling, procedural narrative
- **Music Theory** - Techno composition, evolving music systems
- **Architecture** - Medieval architecture, regional variation
- **Geology** - Realistic terrain formation, geological structures

## Scheduling Strategy

### Sprint-Based Literature Reviews
- **Frequency:** Once per sprint (e.g., weekly or bi-weekly)
- **Duration:** 1-2 days per sprint dedicated to literature review
- **Assignment:** Credibility Analyst + Creative Filter + Emergent Effects Researcher
- **Output:** Wiki entries, backlog items, or direct design proposals

### Domain Rotation
- Each sprint focuses on 1-2 primary domains
- Rotate through domains systematically
- Allow for hot-pursuit of interesting findings (if something great is found, dive deeper)

### Example Sprint Schedule

**Sprint 1:** Game Design + Simulation Theory
- Credibility Analyst: Reviews recent game design papers
- Creative Filter: Looks for transferable mechanics from other fields
- Emergent Effects Researcher: Studies emergent systems in nature/computation

**Sprint 2:** Economics + Psychology/Sociology
- Credibility Analyst: Reviews behavioral economics papers
- Creative Filter: Looks for economic models applicable to games
- Emergent Effects Researcher: Studies social emergence (crowds, movements)

**Sprint 3:** History + Theology
- Credibility Analyst: Reviews medieval history sources
- Creative Filter: Looks for historical mechanics (e.g., guild systems)
- Emergent Effects Researcher: Studies religious emergence (cults, movements)

**Sprint 4:** Pharmacology/QSP + Biology
- Credibility Analyst: Reviews QSP models and applications
- Creative Filter: Looks for QSP applications to non-medical game systems
- Emergent Effects Researcher: Studies biological emergence (ecosystems, evolution)

**Sprint 5:** Hot Pursuit (Flexible)
- Based on previous findings, dive deeper into most promising areas
- Could be multiple domains if interdisciplinary connections found

## Literature Review Process

### 1. Planning Phase (Manager Bot)
- Select domains for this sprint's literature review
- Assign tasks to research agents
- Define search terms and sources
- Set expectations for output (wiki entries, backlog items, design proposals)

### 2. Research Phase (Research Agents)
**Credibility Analyst:**
- Search academic databases (Google Scholar, arXiv, JSTOR, etc.)
- Find high-quality, peer-reviewed sources
- Validate credibility of sources
- Summarize key findings
- Assess relevance to game project

**Creative Filter:**
- Review findings from Credibility Analyst
- Look for simple mechanics that can be borrowed
- Identify cross-domain applications (e.g., QSP for magic systems)
- Propose creative adaptations
- Look for "low-hanging fruit" - easy to implement, high impact

**Emergent Effects Researcher:**
- Review findings for emergent systems
- Look for papers on emergence, complexity, bottom-up organization
- Propose emergent mechanics for the game
- Write tests or agents to prove out ideas
- Structure ideas as modular components

### 3. Synthesis Phase (Manager Bot + Research Agents)
- Combine findings from all research agents
- Identify connections between domains
- Prioritize findings by:
  - Relevance to current development needs
  - Novelty (is this something new?)
  - Feasibility (can we actually implement this?)
  - Impact (how much would this improve the game?)
- Decide on output format:
  - Wiki entry (for reference)
  - Backlog item (for future implementation)
  - Design proposal (for immediate consideration)
  - Discard (not relevant)

### 4. Integration Phase
- Create wiki entries for relevant findings
- Add backlog items for promising mechanics
- Present design proposals to user for consideration
- Update research agents' KBs with learnings
- Update Manager Bot's KB on what research areas are fruitful

## Output Formats

### Wiki Entry
Used for: Reference material, background knowledge, academic sources

**Template:**
```markdown
# [Research Topic]

**Source:** [Paper/Book/Article]
**Authors:** [Author names]
**Year:** [Year]
**URL/DOI:** [Link]
**Domain:** [Game Design / Economics / Psychology / etc.]

## Summary
[Brief summary of the research]

## Key Findings
- [Finding 1]
- [Finding 2]
- [Finding 3]

## Relevance to Game Project
[How this applies to our game]

## Potential Applications
- [Application 1]
- [Application 2]

## Further Reading
- [Related sources]
```

### Backlog Item
Used for: Mechanics to implement, systems to build, research to do deeper

**Template:**
```markdown
### [ ] [Research-Derived Item]
- **Status:** Backlog
- **Priority:** High/Medium/Low
- **Category:** software-design/artistic-references/etc.
- **Added:** [Date]
- **Source:** [Paper/Book that inspired this]
- **Notes:** [What needs to be implemented/researched]
```

### Design Proposal
Used for: Immediate consideration for current development

**Template:**
```markdown
# Design Proposal: [Proposal Title]

**Source:** [Research finding]
**Proposed By:** [Agent]
**Date:** [Date]

## Summary
[Brief description of proposal]

## Research Basis
[What research inspired this proposal]

## Proposed Implementation
[How this would work in the game]

## Pros
- [Advantage 1]
- [Advantage 2]

## Cons
- [Disadvantage 1]
- [Disadvantage 2]

## Effort Estimate
[How much work would this be?]

## Recommendation
[Should we do this? Why or why not?]
```

## Search Strategies

### Academic Databases
- **Google Scholar** - Broad search, good for finding related work
- **arXiv** - Pre-prints, cutting-edge research (CS, physics, math, etc.)
- **JSTOR** - Humanities and social sciences
- **PubMed** - Medical and biological research
- **IEEE Xplore** - Engineering and computer science
- **ACM Digital Library** - Computer science

### Search Terms by Domain

**Game Design:**
- "emergent gameplay" "procedural generation" "player agency" "game design patterns"
- "narrative simulation" "agent-based modeling games" "complexity in games"

**Simulation Theory:**
- "agent-based modeling" "complex adaptive systems" "emergent behavior"
- "cellular automata" "fractal generation" "bottom-up organization"

**Economics:**
- "behavioral economics" "lifespan economics" "age-based preferences"
- "alternative economic models" "resource scarcity economics"

**Psychology/Sociology:**
- "attachment theory" "social dynamics" "group behavior"
- "influence systems" "community organizing" "social networks"

**Pharmacology/QSP:**
- "quantitative systems pharmacology" "ADME modeling" "physiologically-based pharmacokinetics"
- "disease progression modeling" "pharmacodynamics"

**History:**
- "medieval economic history" "printmaking history" "tea trade history"
- "textile production history" "guild systems medieval"

**Theology/Philosophy:**
- "Dante Inferno cosmology" "religious emergence" "cult formation"
- "ethical frameworks" "moral decision making"

### Evaluation Criteria for Sources
1. **Credibility** - Peer-reviewed? From reputable source? Author credentials?
2. **Relevance** - Directly applicable to game design or transferable?
3. **Novelty** - Is this new information or already known?
4. **Quality** - Well-researched? Good methodology? Clear findings?
5. **Recency** - For fast-moving fields (CS, AI), prefer recent. For history, older is fine.

## Managing Research Findings

### Tracking System
- Each literature review sprint creates a summary document
- Summary includes: domains researched, sources found, key findings, actions taken
- Stored in `wiki/literature-reviews/[sprint-number].md`

### Example Summary
```markdown
# Literature Review Summary - Sprint 5

**Date:** 2026-10-25
**Domains:** Game Design + Simulation Theory
**Agents:** Credibility Analyst, Creative Filter, Emergent Effects Researcher

## Sources Reviewed
1. "Emergent Narrative in Simulation Games" - Smith et al., 2023 (arXiv)
2. "Agent-Based Modeling of Social Networks" - Johnson, 2022 (JSTOR)
3. "Fractal Generation in Procedural Content" - Lee & Kim, 2024 (IEEE)
... [total 15 sources]

## Key Findings
1. [Finding 1 from game design]
2. [Finding 2 from simulation theory]
...

## Actions Taken
- Created wiki entry: "Emergent Narrative Research"
- Added backlog item: "Social Network Simulation"
- Design proposal: "Fractal Terrain Generation" - presented to user
- Discarded: 3 sources not relevant

## Next Steps
- User approved design proposal for fractal terrain - add to next sprint
- Social network simulation added to backlog - prioritize based on tech tree needs
- Consider deeper dive into emergent narrative in future sprint
```

### Prioritization Framework
For each finding, rate on:
- **Relevance (1-5)** - How applicable to our game?
- **Novelty (1-5)** - How new is this to us?
- **Feasibility (1-5)** - How easy to implement?
- **Impact (1-5)** - How much would this improve the game?

**Score = Relevance × Novelty × Feasibility × Impact**
- Score > 50: High priority, implement soon
- Score 25-50: Medium priority, add to backlog
- Score < 25: Low priority, maybe discard

## Integration with Agent Workflow

### Literature Review as Sprint Task
- Manager Bot includes literature review tasks in sprint plan
- Assigned to research agents as part of sprint
- Timeboxed (e.g., 1-2 days per sprint)
- Output integrated into sprint review

### Research Agent KB Updates
- After literature review, research agents update their KBs:
  - Credibility Analyst: `sources-reviewed.md`, `credibility-assessments.md`
  - Creative Filter: `creative-ideas.md`, `cross-domain-applications.md`
  - Emergent Effects Researcher: `emergent-systems.md`, `proven-ideas.md`

### Manager Bot Learning
- Manager Bot tracks which domains yield useful findings
- Adjusts future literature review scheduling based on productivity
- Example: "Game design research has yielded 5 useful findings, theology has yielded 0. Shift focus."

## Hot Pursuit Mode

### Trigger
- When a particularly promising finding is discovered
- User requests deeper dive into specific area
- Multiple findings suggest interdisciplinary opportunity

### Process
- Pause domain rotation
- Dedicate next sprint (or part of sprint) to deep dive
- Multiple research agents may collaborate on single domain
- Goal: Exhaustive research, not just survey

### Example
**Finding:** "QSP models show interesting emergent behavior in drug interactions"
**Hot Pursuit:** Dedicate Sprint 6 to QSP deep dive
- Credibility Analyst: Comprehensive QSP literature review
- Creative Filter: All possible QSP applications to game systems
- Emergent Effects Researcher: QSP as emergent system framework
- Output: Detailed QSP integration proposal, possibly multiple design proposals

## User Involvement

### Review Opportunities
- After each literature review sprint, Manager Bot presents summary
- User can:
  - Approve/reject design proposals
  - Request deeper dive into specific findings
  - Suggest new domains to research
  - Adjust scheduling (frequency, duration)

### Direct Requests
- User can request specific literature review at any time
- Example: "I want to know more about attachment theory in games"
- Manager Bot schedules special literature review task
- May interrupt normal domain rotation

### Feedback Loop
- User feedback on literature review quality informs future scheduling
- If user finds reviews useful, continue or expand
- If user finds reviews not useful, adjust domains or approach

## Metrics

### Success Metrics
- **Findings Implemented** - How many literature-derived features get implemented?
- **User Satisfaction** - Does user find literature reviews valuable?
- **Novelty Score** - Are findings genuinely new to the project?
- **Time to Impact** - How long from discovery to implementation?

### Process Metrics
- **Sources per Sprint** - How many sources reviewed per literature review sprint?
- **Relevance Rate** - What percentage of sources are relevant?
- **Action Rate** - What percentage of findings result in action (wiki/backlog/proposal)?
- **Domain Productivity** - Which domains yield most useful findings?

## Challenges and Mitigations

### Challenge: Too Much Information
**Mitigation:** Strict filtering by Credibility Analyst, prioritization framework, timeboxing

### Challenge: Research Not Actionable
**Mitigation:** Creative Filter focuses on simple, transferable mechanics; Manager Bot prioritizes by feasibility

### Challenge: Academic Jargon
**Mitigation:** Credibility Analyst summarizes in plain language; Creative Filter translates to game terms

### Challenge: False Positives (Research that sounds good but doesn't work)
**Mitigation:** Emergent Effects Researcher writes tests/agents to prove out ideas before implementation

### Challenge: Literature Review Takes Time from Implementation
**Mitigation:** Timeboxed to 1-2 days per sprint; consider whether value justifies time; adjust based on metrics

## Initial Schedule

### Sprint 1 (First Sprint)
Skip literature review - focus on setting up basic infrastructure

### Sprint 2
**Domains:** Game Design + Simulation Theory
**Rationale:** Foundational for understanding how to build the game

### Sprint 3
**Domains:** Economics + Psychology/Sociology
**Rationale:** Core simulation systems

### Sprint 4
**Domains:** History + Theology
**Rationale:** Thematic and narrative foundation

### Sprint 5
**Review:** Assess productivity of first 4 literature review sprints
**Decision:** Continue current schedule? Adjust domains? Change frequency?

### Ongoing
- Continue literature review each sprint (unless metrics show not valuable)
- Rotate through domains systematically
- Allow for hot pursuit when promising findings emerge
