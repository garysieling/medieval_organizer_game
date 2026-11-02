# Campaign Dialogue Trees: Staged Content and Integration Plan

**Status:** Stages 0-1 merged; Stage 2 (1,000 house-meeting beats) in review; Stages 3-4 pending
**Branch:** `feat/campaign-dialogue-trees`
**Reference model:** `game2-core/src/game/conversation.rs` and `game2-core/src/game/dialogue_trees.rs`

## Goal

Give campaign organizing the same contextual, weighted, authored decision model as person-to-person conversation. The content should make house meetings feel like consequential rooms with uneven power, competing stakes, and changing participation; campaign arcs should feel like strategic action/reaction loops that can win, lose, settle partially, learn, and continue into enforcement or renewed opposition.

This is not a request for 2,000 interchangeable quotations. Each counted entry is a distinct, authored decision beat with a stable ID, context criteria, weighted response choices, dialogue text, and a declared transition or consequence hook. Each choice can contain several interchangeable line variants without inflating the entry count.

## Content Target and Entry Contract

Deliver two separately counted libraries:

1. **House-meeting library: 1,000 decision beats.** A medium-scale group deliberation: agenda setting, who gets the floor, testimony, proposal, amendment, objection, coalition-building, vote/consent, and follow-through. Include quiet influence, formal office, turnout, dissent, faction, broker roles, scarcity, fear, trust, and unequal exposure to risk.
2. **Campaign-arc library: 1,000 decision beats.** A large-scale action/reaction cycle: define a target and demand, build capacity, test participation, escalate or pause, answer counter-moves, negotiate, record full/partial wins and losses, protect gains, learn, and choose the next campaign.

An entry must have: unique stable ID; library and phase; one or more matching criteria; one or more options with positive base weights; dialogue/action text; consequence/transition; and source-pattern tags. Add authored weights as relative likelihoods, not claims that history has a universal measurable probability. Runtime context, relationships, traits, stakes, institutional position, resources, fatigue, discipline, and previous outcomes may bias weights while retaining viable alternatives. Outcomes such as consent, repression, or a full victory are not guaranteed by selecting a line.

## Historical and Voice Scope

Use the existing organizing research corpus as the first source map: historical campaigns, premodern resistance, religious institutions, anticolonial campaigns, campaign design, internal structure, information networks, countertactics, enforcement, and modern counter-playbooks. Add examples across a broad chronology, from the Exodus tradition and ancient civic/religious associations through medieval guilds, communes, peasant resistance and religious movements, early-modern petitions and mutual aid, abolition and labor, suffrage and civil rights, anticolonial struggles, tenant/farmworker campaigns, disability and climate organizing, and contemporary institutional campaigns.

Distinguish sacred narrative, legendary accounts, contested causality, and well-documented historical cases instead of presenting them as equivalent evidence. Transfer mechanisms (solidarity, coordination, framing, escalation, bargaining, backlash, institutional durability), not exact modern institutions or slogans into the game's medieval setting. Preserve the project's established villager voice: concise, concrete, period-compatible language; avoid accidental shifts between archaic and modern diction. Keep authored text distinct from structural labels and source notes.

## Model and Integration Direction

Follow the conversation framework's useful properties: declarative `Has`/`Lacks`-style context, specificity-first matching with a generic fallback, weighted alternatives, concepts for threading discussion, and reusable line variants. Do not copy person-to-person `Act` semantics blindly. Define meeting and campaign acts/phases appropriate to collective decisions and outcomes.

The campaign implementation currently has procedural founding votes and meeting outcomes in `game2-core/src/game/campaign.rs`; regular conversation data lives in `dialogue_trees.rs`. The first integration should add a campaign-dialogue content/model boundary and connect one narrowly scoped decision loop before extending the full lifecycle. Preserve deterministic seeded behavior and provide a safe generic fallback when no specific content matches. Content selection must remain separate from world-state mutation; a chosen consequence is applied through explicit, testable hooks.

## Stages and Checkpoints

### Stage 0 — Plan (this checkpoint)

- Record the two-library target, historical/voice policy, entry contract, integration direction, and staged verification.
- Check in and publish this plan before large-scale implementation.

### Stage 1 — Schema and representative slice

- Define the data model for criteria, weighted options, concepts, text variants, transitions, and consequence hooks.
- Build and validate a representative tranche of 50 meeting beats and 50 campaign-arc beats across distinct contexts and phases.
- Add inventory tooling/tests for unique IDs, required fields, valid positive weights, non-empty fallback coverage, and resolvable transitions/hooks.
- Connect one meeting decision path to the model and prove deterministic selection with seeded tests.
- Review voice consistency and ensure historical source-pattern labels are discoverable.

**Gate:** approve schema ergonomics, authored language, probability semantics, testability, and runtime boundary before bulk generation.

### Stage 2 — House meetings to 1,000

- Expand by meeting phase and power dynamic, not by superficial paraphrase.
- Include decisions over agenda, speaking order, evidence, demand wording, amendment, coalition, dissent, consent/majority, leadership accountability, records, and next steps.
- Track distinct contexts and meaningful branch/consequence coverage; reject duplicates that differ only by names or punctuation.
- Validate counts, IDs, fallback, weights, transitions, voice, and a seeded sample of actual selection.

### Stage 3 — Campaign arcs to 1,000

- Expand across build, mobilize, campaign, win/lose, enforce, defend, and restart phases.
- Include petitions, structure tests, collective action, negotiation, countertactics, full/partial wins and losses, concessions, broken promises, repression/backfire, coalition shifts, leadership succession, and learning.
- Model leader demands as situated options: specific/winnable, broader/transformational, procedural, material, protective, and accountability demands. Outcomes can be granted, denied, delayed, diluted, or split.
- Validate that wins are not terminal by default and losses can produce learning without erasing costs.

### Stage 4 — Runtime integration and balancing

- Wire meeting and arc selection to relevant campaign state, relationships, status, power dynamics, prior choices, and consequence hooks.
- Keep choice weights understandable and tuneable; add deterministic tests for specificity, contextual bias, fallback, and consequence application.
- Run focused campaign/meeting tests and a long-run playtest/balance pass; inspect repeated-line rates, dead ends, dominant choices, phase coverage, and stale campaign loops.
- Document authoring and validation commands, summarize source coverage, and publish the completed PR for review.

## Acceptance Criteria

- Exactly 1,000 valid, unique house-meeting decision beats and 1,000 valid, unique campaign-arc decision beats, reported separately by automated inventory checks.
- Choices use contextual matching and authored positive base weights, with a generic fallback and deterministic seeded selection.
- Each entry has meaningful dialogue/action text plus an explicit transition or consequence hook; a data audit shows no unresolved hooks or unreachable phases.
- Coverage includes power imbalance, informal/formal leadership, demands, partial outcomes, failure, backlash, enforcement, and campaign renewal.
- Text is consistent with the game's register; research notes distinguish tradition, evidence strength, and transferable mechanism.
- Existing conversation behavior and unrelated campaign mechanics remain unchanged outside the explicitly tested integration points.

## Risks and Decisions

- **Content scale:** 2,000 distinct decision beats can become repetitive. Stage gates and duplicate/context audits are mandatory; count structural choice situations, not variants.
- **Probability overclaim:** weights are game-authored priors, not historical frequencies. Keep them contextual and tunable.
- **Scope creep:** this work supplies dialogue/decision content and hooks. It should not silently add an expansive new power simulation; use existing campaign state and clearly surface missing state as follow-up work.
- **Historical compression:** avoid implying that unlike movements used identical tactics or had simple, uncontested outcomes. Tag source patterns and represent uncertainty in research notes rather than in-character exposition.
- **Branch base:** start from local `main` at `f540bed`, which contains the recent conversation framework and dialogue expansion; do not modify the user's dirty checkout.

## Initial Local Hypothesis and Discriminating Check

The likely integration boundary is a declarative campaign decision library beside the conversation library, with explicit consequence hooks into `campaign.rs`; the current meeting function itself resolves attendance and campaign bookkeeping but does not yet model a deliberative exchange. Stage 1's seeded selection test and a narrow integration test should disconfirm this if the existing campaign system already exposes a suitable decision abstraction or if content selection cannot be kept separate from state mutation.
## Stage 1 Notes

- Model, selection, runtime hooks and the inventory audit: `game2-core/src/game/campaign_dialogue.rs`. Content and source patterns: `game2-core/src/game/campaign_beats.rs`.
- A beat is the counted unit (stable ID, phase, criteria, voice, prompt variants, weighted moves, source tags). Moves carry criteria, base weight, fact-keyed bias, a voice, line variants, one `Hook`, and a `Next` phase or close.
- Selection: most-specific tier among unspent matching beats (seeded tie-break), then a weighted move draw. Every phase keeps a criteria-free beat; every beat keeps a criteria-free move.
- Hooks are the only mutation path: `Resolve` (+1 resolve on record; requires a note taker), `Dissent` (speaker rejoins dissenters; never the org leader), `Reframe(Framing)`.
- Connected path: after `hold_campaign_meeting`, `deliberate_meeting` walks the house-meeting library from `Agenda` (max 5 beats) against facts read from the campaign, voices lines through attendees, and applies hooks. `hold_campaign_meeting` itself is unchanged.
- `arc:*` facts are defined vocabulary only; the campaign-arc library is audited and walkable in tests but not yet wired to state (Stage 3/4).
- Audit: `cargo test -p game2-core --lib game::campaign_dialogue` checks counts per library, unique IDs, positive weights, known facts, resolvable source tags, phase fallbacks, reachability, voice/hook backing, distinct in-register lines, and seeded determinism.

## Stage 2 Notes

- The house-meeting library moved to `game2-core/src/game/campaign_meeting_beats.rs`: 1,000 beats (agenda 100, floor 100, testimony 140, proposal 129, amendment 100, objection 130, coalition 100, decision 120, follow-through 81). `campaign_beats.rs` keeps the arc library, source patterns (now 38, adding sacred, legendary and contested cases alongside the documented ones) and the `bt`/`mv` authoring helpers.
- New runtime facts read by `meeting_facts` (now given the tick): `room:trait:*` (any attendee's temperament), `room:want:*` (what attendees' live problems leave them short of: bread, relief, justice, roof, work, mourning), `room:faction` (two or more heard dissenters), `org:jailed`, `org:weak`/`org:strong` (organized-power capacity below 0.3 / at least 1.0), `org:leader_absent`. `EXCLUSIVE_FACTS` names facts that can't co-occur.
- Audits added: exact per-library counts with a per-phase floor, satisfiable criteria, at least 450 distinct phase+criteria contexts (558 at merge), every runtime fact used by at least 3 beats, at least 8 criteria-free beats per phase, dissent/reframe/close coverage, and a seeded 6,000-room sample that must reach 90% of the library (991/1,000 at merge).
- Authors can read play-throughs with `cargo test -p game2-core --lib sample_meetings -- --ignored --nocapture`.
- Content note: a beat whose criteria are mostly `Lacks` still scores its full criterion count, so it outranks every generic beat in most rooms. Pair a `Lacks` with a `Has` that narrows it.
