# Lit Review - Sprint 7: Organizing Power - "Power Recognizes Power" and "Power Builds as You Spend It"

**Date:** 2026-10-25
**Domain:** Political Science / Community Organizing / Sociology
**Backlog items:** Community Organizing (power recognizes power; power builds as you spend it); also Relationships over time, Injustices & labor organizing, Vocation-driven relationships
**Sources:** McAlevey *No Shortcuts: Organizing for Power in the New Gilded Age* (Oxford 2016); Alinsky *Rules for Radicals* (1971); Ganz "Public Narrative, Collective Action, and Power" (World Bank 2011) and *People, Power, Change* ch. "Strategizing" (Oxford 2024); Han *How Organizations Develop Activists* (Oxford 2014). Builds on the Sprint 3 community-organizing and Sprint 4 organizing-leadership reviews.

## Sources

### 1. McAlevey - *No Shortcuts* (2016)
- Three change models: **advocacy** (experts act for people), **mobilizing** (activists turn out the already-committed), **organizing** (bring in new people, especially "organic leaders", to build lasting power)
- Only deep organizing produces high-impact wins; mobilizing gets short, shallow victories
- **Power structure analysis:** map who holds power over the decision **and** the strength of your own side, then act
- **Structure tests:** visible public actions (majority petitions, mass sign-ups) that **prove** strength before a strike; they test you *and* signal to the target
- Workplace fights are deeply about **dignity**, not just material gain
- **Relevance:** CRITICAL - structure tests are literally "power recognizes power": the boss bargains only once he sees a supermajority

### 2. Alinsky - *Rules for Radicals* (1971)
- "Power has always gravitated toward those who have money and those whom people follow"; the have-nots' resource is **numbers**
- Rule 1: "**Power is not only what you have but what the enemy thinks you have**", so perceived power counts
- Compromise is a key word: "If you start with nothing, demand 100 percent, then compromise for 30 percent, you're 30 percent ahead"
- The organizer must earn a "**license to operate**" from the community's own experience
- Opponents labeled him a "public menace", and that recognition was itself a marker of power
- **Relevance:** CRITICAL - **reputational power** is distinct from real power; targets react to perceived strength

### 3. Ganz - Public Narrative + Strategizing (2011; 2024)
- Strategy = "turning what you **have** (resources) into what you **need** (power) to get what you **want** (change)"
- Power is **relational**: it comes from interdependence (who depends on whom for what), not a stockpile
- Public narrative (story of self / us / now) turns values into action and builds shared identity
- United Farm Workers example: a resource-poor constituency won through strategic capacity
- **Relevance:** CRITICAL - power is a property of relationships, so it fits the pairwise relationship model

### 4. Han - *How Organizations Develop Activists* (2014)
- **Transactional mobilizing** (asks as one-off transactions; capacity is used up) vs **transformational organizing** (each ask *develops* the person: skills, relationships, motivation)
- High-engagement organizations give people **real responsibility** and **reflection/coaching**; involvement **increases** future involvement
- **Relevance:** CRITICAL - direct evidence for "power builds as you spend it": using people's capacity well grows it

## The Two Concepts as Mechanics

### A. Power recognizes power
- Every actor (ruler, guild, lord, org) holds a **perceived power** estimate of every other org it deals with
- Negotiation/attention thresholds: a target **ignores** demands below its threshold, **negotiates** above it, **represses or concedes** far above
- Perceived power updates from **visible actions** (structure tests): marches, petitions, successful strikes, the size of the crowd at the bridge
- Perception ≠ reality (Alinsky): bluffing works short-term; getting caught bluffing crashes perceived power
- Recognition is mutual and **status-conferring**: being attacked or negotiated with by the powerful raises your standing with others

### B. Power builds as you spend it
- Model org power as a **capacity that grows with skillful use** (like a muscle or the road-use loop), not a currency that depletes:
  - `capacity += growth * successful_actions * development_quality`
  - `capacity -= atrophy * idle_time`
  - failed or over-reaching actions cost capacity (burnout, demoralization)
- **Transactional vs transformational asks** (Han): the same action either **consumes** member commitment (transactional) or **grows** it (transformational: real roles, reflection, relationship building)
- Each action builds **relationships** (pairwise ties strengthen when people act together), and relationships *are* the stored power (Ganz)
- Contrast: money/gold power **depletes when spent**; people power **compounds when spent well**. That asymmetry is the core gameplay contrast between oligarch and organizer factions (ties to the medieval allegory doc)

## Design Proposals

1. **Two power currencies:** `wealth` (depletes on spend) vs `organized_power` (grows on good use, atrophies when idle). Different factions lean on different ones
2. **Perceived-power matrix** between orgs, updated by visible actions; drives whether demands are ignored, negotiated, or repressed
3. **Structure test actions:** petition, mass meeting, slowdown. Low-risk shows of strength that raise perceived power and reveal actual commitment before a high-risk action (strike, blockade)
4. **Organizer quality** as a skill: transformational organizers grow members' capacity per action; transactional ones burn it
5. **Org-voted actions** (from the labor organizing backlog item) consume/grow power by these rules; overreach without a structure test risks collapse
6. **Relationship coupling:** shared actions strengthen pairwise ties faster when people share org identity (Relationships-over-time item)

## Rating
**Critical** - these two rules are the strategic heart of the organizer-vs-power allegory. They give a principled reason why people power behaves differently from money power, and both map onto existing relationship and reputation systems (`sim/community.rs`, `sim/reputation_qsp.rs`).
