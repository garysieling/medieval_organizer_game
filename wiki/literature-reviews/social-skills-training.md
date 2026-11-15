# Social-Skills Training Literature Review

**Survey Date:** 2026-11-14
**Issue:** #1210
**Agent:** Literature Survey Agent
**Confidence:** High
**Domain:** Social-skills pedagogy (ADHD/autism curricula, conversation repair)

## Summary

The social-skills training literature decomposes conversation into *explicit, teachable rules* — precisely the vocabulary our declarative dialogue system already speaks. PEERS (UCLA, Laugeson) teaches conversation as a named sequence — find a common interest → mention it → trade information → assess interest → graceful entry/exit — and catalogs named failure modes (the conversation hog, the missed exit cue). Skillstreaming breaks each behavior into numbered steps. Social Thinking frames everything as expected/unexpected. For us the value is two-sided: confirmation that our trade-and-bid machinery mirrors how conversation is *actually taught*, and a ready-made vocabulary of failure modes and repair moves we lack.

## Key Sources

### Laugeson & Frankel — *PEERS: Social Skills for Teenagers* / UCLA PEERS Program (2012+)
**Rating:** A
**Type:** Evidence-based clinical curriculum (RCT-validated)
**Relevance:** Direct — conversation decomposed into named rules and failure modes

**Key Findings:**
- **Trading information** is the core rule: find a common interest → mention it (comment/question/compliment) → trade info back and forth → follow-up questions keep the topic → *answer your own question if they don't ask back* → assess interest (looking? facing? talking?) → then introduce.
- **Friendships are based on common interests** — the named failure mode is the *conversation hog*: one who talks their own interest without trading or reading signals.
- **Peer entry** is assessed before joining (receptiveness read); **peer exit** has its own steps (excuse line + depart).
- **Repair is its own lesson**: changing a bad reputation is taught as a *long-term deliberate strategy*, and "resolving arguments" has numbered problem-solving steps.
- Rumor/gossip damage control is its own module: minimize spread, don't feed it.

### McGinnis & Goldstein — *Skillstreaming* series
**Rating:** A−
**Type:** Structured learning curriculum (schools/clinical)
**Relevance:** The skill-steps format — each social act as an ordered checklist

**Key Findings:**
- Every behavior decomposes to 4–6 numbered steps taught explicitly, modeled, then rehearsed with feedback — conversation is *built*, not intuited. Validates the declarative-rule approach: our `Act` taxonomy (Greet/Remark/Question/Answer/Agree/PushBack/Farewell) is exactly the granularity these curricula teach at.

### Michelle Garcia Winner — Social Thinking methodology
**Rating:** B+
**Type:** Practitioner framework (widely adopted, debated)
**Relevance:** The expected/unexpected framing — behavior judged relative to context and the other's comfort

**Key Findings:**
- Behavior is *situationally* right or wrong: the same act reads expected/unexpected by context, and the other's feelings about you shift accordingly. That's our `TalkMood` + trait-weighted reactions: what you said lands inside who said it, where, and to whom.

### Chris MacLeod — *The Social Skills Guidebook* (2016)
**Rating:** B
**Type:** Self-help manual for socially anxious adults
**Relevance:** The civilian version — same rules (topic trade, listening ratio, graceful exits) written for general readers; confirms the failure catalog

## Game Mechanic Proposals

### Mechanic 1: The Amends — a wrong word acknowledged → **#1251**
**Basis:** PEERS' repair lessons + Skillstreaming's conflict-resolution steps.
**Translation:** A talk option that appears only on a Slighted `TalkMemory` (or a fresh `Conflict` interaction): acknowledge the wrong, spend the beat, take the cool answer — partial chill relief, mood to guarded; refusals feed the grudge layer.
**Touches:** `dialogue_trees.rs` (amends arc), `conversation.rs` (option gating on memory), `relationships.rs` (partial reversal).

### Mechanic 2: The Hog — a villager who never trades → **#1252**
**Basis:** PEERS' named failure mode.
**Translation:** A trait whose bearers weight their own pools and near-never ask back: monologue talks, fewer reveals but richer self-disclosure, long bids at meetings the room pushes back on. The PEERS exit-misser (doesn't read your farewells) rides the same texture.
**Touches:** `traits.rs` (trait), `conversation.rs` (reply weighting), `meetings.rs` (bid weight).

### Not proposed (and why)
- **Trading information / common interests:** the bid-and-concept machinery already runs exactly this — PEERS' core rule is our engine's behavior.
- **Peer entry assessment:** the vibe probe (#1187) is literally the receptiveness read.
- **Exit lines:** farewells + talked-dry endings cover it (and #1252 proposes the *failure* to read one).
- **Humor feedback loops:** too thin for a system — folds into trait-flavored lines if ever wanted.

## Next Steps
- #1251, #1252 enter the normal queue.
- Adjacent filed work: #1232 (daily-greeting rule) intersects the amends spam-guard question.
