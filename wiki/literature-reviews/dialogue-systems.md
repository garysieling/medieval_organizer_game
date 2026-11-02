# Lit Review - Dialogue Systems: Rule Databases, Beats, and Social Physics

**Date:** 2026-11-01
**Domain:** Game AI / Interactive Narrative
**Backlog items:** Conversation framework - two-character call/response trees with weighted forks and specificity overrides; house meetings with concept-tagged threading
**Sources:** Ruskin "Rule Databases for Contextual Dialogue and Game Logic" (GDC 2012); Mateas & Stern, Façade architecture papers (CMU-CS-02-198; AIIDE 2005; TIDSE 2003); McCoy et al., Prom Week / Comme il Faut (T-CIAIG 2014; FDG 2011); Evans & Short, Versu (2014)

## Sources

### 1. Ruskin - "Rule Databases for Contextual Dialogue and Game Logic" (Valve, GDC 2012)
- The Left 4 Dead / DOTA 2 dialogue engine: a **rule database**, not a branching script. Each rule = a set of **criteria** (functions on world state) + a **response** (the line)
- When a character should speak, the engine builds a **query** = a pile of key-value **facts** about the world (map, health, who's dead, what just happened), then finds rules whose criteria all match
- **Scoring = specificity**: the rule with the *most criteria* wins. A "zombie on the merry-go-round next to the ice-cream machine" line beats a generic "oh look a zombie" line - automatically
- Generic rules double as fallback: if nothing specific matches, the least-specific matching rule still fires, so the character *always has something to say*
- **Relevance:** CRITICAL - this is literally the proposed "most specific tree wins, fall back to generic" mechanism, proven at Valve scale. Also removes the need to author branch structure: specificity ordering makes the structure *emerge*

### 2. Façade - Mateas & Stern (interactive drama, 2002-2005)
- Drama decomposed into **beats**: bundles of behaviors around a topic, with preconditions and effects; a **drama manager** sequences beats for well-formed pacing while the moment-to-moment stays simulated
- Atomic unit = **joint dialog behavior (jdb)**: a 1-5 line coordinated exchange between characters
- Player input parsed into **discourse acts** (agree, disagree, thank, criticize, flirt...) rather than literal text - characters respond to the *act*, not the words
- **Relevance:** HIGH - discourse acts are how replies stay coherent without scripting every pair; "respond to the act, not the words" is what makes replies feel like they follow. Beats map to our tree levels

### 3. Prom Week / Comme il Faut - McCoy, Treanor, Samuel et al. (UCSC, 2011-2014)
- **Social exchanges**: an action one character takes toward another (flirt, insult, buddy up) + the responder's accept/reject. ~40 exchange types, each accept/reject, each with 20+ templated dialogue scenes
- **Social considerations** (~5000): traits (witty, shy), statuses (angry at X), relationships, and cultural affinities each *nudge* the desirability of an exchange rather than hard-gating it. Sometimes people are mean to friends - the model allows it, just weights it
- Responder evaluates independently: desire to act and willingness to accept are separate rolls
- **Relevance:** CRITICAL - this is the "personality biases the forks" mechanism: traits/status multiply response weights rather than filter them. Accept/reject split = our call/responder turn structure

### 4. Versu - Evans & Short (2014)
- Characters in **social practices** pick **quips**: dialogue acts with **prerequisites** (must have happened/not happened) and **exclusivities** (can't say X and Y in the same practice)
- Quip choice is utility-scored per character per practice; content is authored as a pool, not a fixed script
- **Relevance:** HIGH - prerequisites = our criteria; exclusivity solves "don't repeat yourself" without tracking raw history; the once-only quip is how a conversation *runs out of material* and naturally ends in small talk

### 5. Authoring practice - branching trees vs databases (Radiator/debacle.us commentary; industry consensus)
- Branching trees are the *player's mental model*, not a required authoring format: 100 condition-gated events that fire on world state produce the *experience* of branching without the authoring cost
- Hub-and-spoke (BioWare) trees need explicit loop-backs; rule databases get loops for free because the smalltalk rules never stop matching
- **Relevance:** MEDIUM - validates the design instinct: author pools + criteria, let the walk be procedural

## The Concepts as Mechanics

### A. Specificity cascade (from Ruskin)
- A **dialogue library** = many trees/pools, each with entry **criteria** (facts about speaker, listener, situation: sick, clergy, rival, grieving, hungry...)
- Sort matching trees by criterion count; walk the most specific first. When it bottoms out (leaf or `end`), **fall through** to the next-most-specific - the sick villager talks about the flu, then drifts to small talk, then says goodbye
- Within a node, reply forks each carry their own criteria: an extra fork can appear only when a fact holds (Hotheaded gets a "snap" option nobody else does) - paths *added* by circumstance, not just reweighted

### B. Weighted forks (from CiF)
- Each reply fork has a base **weight**; the speaker's **traits multiply** it (Hotheaded x3 on sharp replies, Proud x3 on brushing off, Gregarious x2 on questions)
- Pick = weighted random among eligible forks. Personality biases the *distribution*, never hard-decides - people surprise you sometimes, which is what CiF showed makes characters feel human

### C. Act-typed lines + exclusivity (from Façade/Versu)
- Every line carries a **discourse act** (greet, statement, question, answer, farewell) and **concept tags** (what it's about)
- Lines are once-per-conversation (quip exclusivity): when a node offers nothing new, the conversation *degrades* - to small talk, then to farewell. The "stuck in smalltalk, they walk off" ending falls out of material exhaustion, not a special case

### D. Meeting threading (from Façade discourse acts + concept tags)
- A house meeting keeps a **record of statements**; each statement tags its concepts and which earlier statement it replies to
- Each round, candidates bid to speak: **answer the organizer's open question** > **respond to the last speaker** (concept overlap or disagreement) > **revive an earlier statement** (its concepts match their concerns). Recency and overlap score the bids; gregariousness bids more often; whoever just spoke sits out
- The organizer's question stays *open* until answered, so a latecomer can still pick it up three statements later - that's the "reply to the organizer OR the last speaker OR a previous statement" behavior

## Design Proposals

1. `Facts`/`Criterion`/`Reply`/`Exchange`/`DialogueTree` types: criteria gate trees (entry) and forks (intra-node); specificity = criterion count, at both levels
2. `converse(a, b, library, rng)` walks the specificity cascade; a `choose` callback handles the player turn (2 sampled options + "walk away"), so the same engine serves NPC-NPC sim and player-NPC play
3. `run_meeting(...)` produces a `Vec<Statement>` threaded by `replying_to` - renderable as a log and scorable by an AI grader
4. Reuse `game::traits::Trait` for forks' personality bias; reuse `organizing::village_concerns` concept strings as the meeting concept vocabulary
5. Generator binary: N procedurally-built `Speaker`s (name/traits/ideology/status -> generated backstory) x M seeded conversations -> a single markdown artifact for AI grading

## Rating
**Critical** - the proposed architecture is exactly Ruskin's specificity model plus CiF's weighted social considerations, both proven in shipped games. The meeting model is Façade's discourse-act addressing with simpler concept tags. No redesign needed; the literature confirms the shape.
