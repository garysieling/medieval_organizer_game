# Literature Review - Sprint 2: Psychology (Attachment Theory)

**Date:** 2026-10-25
**Domain:** Psychology / Attachment Theory simulation
**Sources Reviewed:** 4 academic works (Dean Petters et al.)

## Sources

### 1. "Modelling emotional attachment: An integrative framework" (2015)
- Petters, IJCNN. 5 citations.
- **Finding:** Organize attachment phenomena into temporally nested scenarios — moment-to-moment behavior inside ontogenetic/phylogenetic sequences. Proposes CogAff schema to integrate models.
- **Relevance:** HIGH - Nested scenario structure maps directly to our "moment → lifecycle" simulation needs.

### 2. "Simulating Infant-Carer Relationship Dynamics" (AAAI 2004)
- Petters. Two competing architectures: deliberative vs reactive.
- **Finding:** 7-part design process for simulating attachment behaviors. Reactive-only architecture can produce plausible attachment behavior.
- **Relevance:** HIGH - Confirms simple reactive agents (like our villagers) can generate psychologically plausible behavior.

### 3. "Attachment Modelling: From Observations to scenarios to designs" (2017)
- Petters & Beaudoin, Springer book chapter.
- **Finding:** 3-stage process: (1) survey behavioral measures (Strange Situation, AAI), (2) abstract observations into scenarios, (3) implement as simulations.
- **Relevance:** HIGH - This IS our process: observe real behavior → abstract to scenarios → implement as sim. Validates our workflow.

### 4. "Implementing a Theory of Attachment: Simulation of the Strange Situation" (2008)
- Autonomous agent sims of infant attachment.
- **Finding:** Infant responses cluster into 3 groups (matching attachment styles). Learning from anxiety episodes produces clustering; learning from warmth doesn't. Explains attachment style emergence at a high level.
- **Relevance:** CRITICAL - Attachment styles can EMERGE from learning mechanism, not be assigned. Fits our emergent design philosophy perfectly. Secure/anxious/avoidant can arise from simple learning rules + anxiety vs warmth episodes.

## Key Insights

1. **Attachment can be emergent** - Styles cluster naturally from learning under anxiety vs warmth. Don't hard-code styles; let them emerge from agent experience.
2. **Scenario-based design** - Petters' observe→abstract→simulate pipeline matches our approach. Use "scenarios" as the unit for psych models.
3. **Reactive agents suffice** - No deliberation needed for attachment behavior. Our ECS agents can produce realistic psych effects.
4. **Nested timescales** - Moment behavior nested in developmental sequences. Design psych systems with multiple timescales.

## Actions

### Backlog Items (concept)
- Attachment style emergence via anxiety/warmth learning episodes (emergent, not assigned)
- Nested-timescale scenario framework for psych models
- "Strange Situation"-style test harness for attachment mechanics

## Rating
**High** - Directly applicable research on simulating psych models with agents. Attachment can be emergent — aligns with our design philosophy.
