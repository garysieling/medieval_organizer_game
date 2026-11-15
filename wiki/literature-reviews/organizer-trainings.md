# Organizer Roleplay Trainings Literature Review

**Survey Date:** 2026-11-14
**Issue:** #1208
**Agent:** Literature Survey Agent
**Confidence:** High
**Domain:** Labor/community organizing pedagogy, relational training

## Summary

The organizing field has a mature, remarkably convergent pedagogy for the exact social situations our game simulates: the one-on-one, the house visit, the first ask, handling push-back, building the committee. Labor Notes' *Secrets of a Successful Organizer* prescribes a seven-beat organizing conversation ending in inoculation against the counter-attack. Marshall Ganz's public-narrative curriculum teaches testimony as a three-story arc (Self → Us → Now). The IAF relational meeting is a scout's tool — a conversation about the person, not the issue, hunting talent and pull. All three treat conversation as a *trainable craft with named beats* — which maps directly onto our talk arcs and meeting structure.

## Key Sources

### Labor Notes — *Secrets of a Successful Organizer* (Bradbury, Brenner, Slaughter, 2016)
**Rating:** A
**Type:** Practitioner manual, canonical in US labor education
**Relevance:** Direct — the scripted beats of the exact conversations we simulate

**Key Findings:**
- The organizing conversation is a fixed arc: **discover the issue** (open-ended questions, 80/20 listening), **agitate** (react — "how does that make you feel?" — giving permission to be angry), **lay the blame** on the decision-maker, **plan to win**, **the hard ask** (a specific commitment), **inoculate**, **agree follow-up**.
- **Inoculation**: ask the worker to predict the boss's counter ("do you think he wants us to organize? what will he do?"), name the playbook (captive-audience meetings, threats, the second-chance plea), rehearse the answer — *then recommit*. The counter-move must land on a prepared member.
- "We tend to remember what we said, not what the other person said" — the asker should get the *worker* to voice the analysis; the ask then references what they themselves said.
- Asks are concrete: sign the card, come Tuesday, wear the sticker — never "support us."

### Marshall Ganz / Leading Change Network — Public Narrative curriculum (Harvard)
**Rating:** A
**Type:** Academic-practitioner curriculum (Harvard Kennedy School)
**Relevance:** The structured arc for testimony/speech-making beats

**Key Findings:**
- Public narrative = **Story of Self** (a *choice* story: challenge → choice → outcome, revealing values, deliberately vulnerable not heroic) + **Story of Us** (shared values/experience, past tense, why we're powerful together) + **Story of Now** (the urgent challenge reframed as a hopeful choice).
- Taught as coachable craft: each story has a shape, a failure mode (heroism, siloed issues, despair), and a rehearsed delivery — testimony is a *skill with a form*, not inspiration.

### Edward T. Chambers / IAF — *Roots for Radicals*, the Relational Meeting
**Rating:** A−
**Type:** Practitioner text (Industrial Areas Foundation method)
**Relevance:** The scouting conversation — surfacing who has pull

**Key Findings:**
- The relational meeting is 30–45 min, face-to-face, *about the person not the issue*: "one organized spirit going after another's spirit," searching for talent, energy, insight.
- Its yield is the social map: who this person is, what drives them, and — read between the answers — **who they and others follow**. Recruitment maps pull, not just sympathy.
- Everything else (caucuses, conventions, actions) only has lasting effect if it grows out of relational meetings.

### IWW Organizer Training 101 / EWOC materials
**Rating:** B+
**Type:** Practitioner trainings
**Relevance:** Confirms convergence — same arc, harder edge (shop-floor mapping, A-B-C worker grading)

**Key Findings:**
- **Charting/mapping**: workers graded on a 1–5 sympathy scale; organizing effort concentrates on the 3s (persuadable middle), 1s become the committee, 5s are worked around. Our stage ladder is the same map drawn differently.
- Roleplay rehearsals score the conversation: did you get their story? did you agitate? did you land a specific ask? — a debrief rubric.

## Game Mechanic Proposals

### Mechanic 1: Inoculation → **#1246**
**Basis:** Labor Notes inoculation step.
**Translation:** A post-commitment talk beat: the organizer names the likely counter (steward's threat, priest's denouncement, the soft plea) and rehearses the answer with the member. The member carries an `inoculated` flag; the next counter-pressure event lands at reduced strength, consumed once.
**Touches:** `dialogue_trees.rs` (inoculate arc), `organizing.rs` (flag), `society.rs` (counter-pressure reads it).

### Mechanic 2: The Self-Said Ask → **#1247**
**Basis:** "We remember what we said" — asks tied to the worker's own stated issue.
**Translation:** Proposition lines template the villager's *revealed* trouble back at them; an ask quoting a fresh known issue gains a yes-bonus. Stale quotes fade.
**Touches:** `dialogue_trees.rs` (issue placeholder in ask pools), `organizing.rs` (`propose`).

### Mechanic 3: Testimony — the Self/Us/Now opening → **#1248**
**Basis:** Ganz's public narrative.
**Translation:** A once-per-campaign meeting opening: the organizer's testimony generated from real game history (their witnessed hanging, their taxing, their cold welcome) in three beats; lifts the room's alignment more than a bare motion.
**Touches:** `campaign_meeting_beats.rs`, `meetings.rs`.

### Mechanic 4: The Scouting Relational Meeting → **#1249**
**Basis:** IAF relational meeting.
**Translation:** A talk path whose yield is the social map: ask who folk turn to; the answer reveals hidden `pull` — who moves the room when they speak. High-pull followers sway meetings; high-pull rivals are marked. Pairs with #1238 (trade-weighted counsel).
**Touches:** `dialogue_trees.rs` (ask-topic), `meetings.rs` (pull-weighted bids).

### Not proposed (and why)
- **80/20 listening, open questions:** already ours — the ask-topics (#1189) are the listen-first mechanic.
- **Charting the 1–5 map:** `stage_of` is the same instrument; the ladder already separates the persuadable middle from the committee.
- **Post-conversation scoring rubric:** the vibe read + TalkMood already debrief the player; a formal score would gamify what should stay felt.

## Next Steps
- #1246–#1249 enter the normal queue for grilling/implementation.
- Related already-filed work: #1230 (typed chat deltas), #1234 (private argument), #1238 (trade-weighted counsel) — all touch the same conversation-mechanics layer.
