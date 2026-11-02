# Lit Review: Inside the Movement — Decision-Making, Meeting Craft, and Training Schools

**Survey Date:** 2026-10-31
**Agent:** Literature Survey Agent
**Domain:** Social Movement Sociology / Participatory Democracy / Movement Pedagogy
**Series:** Organizing research round 2 (user-requested). Feeds `wiki/campaign-plan.md` pieces C1–C3 (founding votes, cadence & roles, the organizing skill tree)
**Confidence:** High — canonical sociology (Snow et al., 6.5k citations), primary movement writing (Freeman), and a fully historied institution (Highlander)

## Purpose

The round-2 surveys covered campaigns against the *outside* (targets, repression, media, enforcement). This one covers the *inside*: how a group of people with a shared grievance becomes a body that decides, meets, delegates, and teaches. Four findings anchor it:

1. **Structurelessness is not neutral** — it doesn't remove power, it hides it (Freeman)
2. **Participation requires frame alignment**, which is an ongoing interactional accomplishment, not a switch (Snow et al.)
3. **The decision rule is itself a design choice** with real trade-offs, not a default (Polletta)
4. **Teaching is how movements multiply** — the Citizenship Schools trained ~100,000 people by training people who trained people (Clark/Horton)

## Finding 1: The Tyranny of Structurelessness (Freeman 1970/72)

Jo Freeman's essay — written from inside women's-liberation rap groups — remains the sharpest statement of the problem:

- **Informal elites are inevitable.** Any group beyond a handful of people forms an elite; an unstructured group's elite is just *informal* — selected by friendship, charisma, or who's always there — and therefore **unaccountable and unremovable**. "For everyone to have the opportunity to be involved in a given group and to participate in its activities the structure must be explicit, not implicit."
- **Structureless groups have a shelf life.** The "rap group" is excellent at consciousness-raising (revelation, bonding — our 1-1s and house meetings) and founders the moment it tries to *do* something specific: "most groups were unwilling to change their structure when they changed their tasks."
- **The fix is not hierarchy, it's accountable delegation.** Freeman's principles: delegate specific authority to specific people for specific tasks by democratic procedure; hold the delegated responsible; distribute and rotate tasks widely; diffuse information through the whole group.

**Mechanic insight:** the note-taker and turnout-coordinator seats aren't administrative flavor — they're the *accountability mechanism*. An org that never delegates develops a **shadow elite**: the member whose personal relationships out-weigh the formal leader's quietly becomes the real center of gravity. Loyalty follows the ties, not the title — and a shadow elite can't be voted out, only split off.

## Finding 2: Frame Alignment (Snow, Rochford, Worden & Benford, ASR 1986)

The canonical micro-mobilization paper: joining a movement is not just self-interest, it's the *linkage of individual and organizational interpretive frames*. Alignment is "a necessary condition for participation, whatever its nature or intensity," and "an interactional and ongoing accomplishment" — it drifts, it's repaired, it's never done.

Four alignment processes, in escalating order of what they do:

| Process | What it does | Campaign-plan analog |
|---|---|---|
| **Bridging** | Links people who already agree but aren't connected | Unscoped issue tags (taxes, poverty, hunger) echo across the whole village — recruitment bridges |
| **Amplification** | Clarifies and invigorates values people already hold | Meetings that voice a member's own stake deepen their commitment — the grumble-fixer |
| **Extension** | Stretches the frame to cover newcomers' interests so "the issue becomes yours too" | The scoped-issue trick: "apprenticeship (mason)" extended to "our children deserve masters" pulls in non-masons |
| **Transformation** | Reinterprets the situation wholesale — the old condition becomes a named injustice | The Framing enum: SharedGrievance → CommonFoe reframes "we're all poor" into "the collector is robbing us" |

**Mechanic insight:** `framing` in `game/campaign.rs` is already a fit-check (`framing_suits` → grumbles); frame alignment gives it teeth. Participation (turnout, strike votes, standing a role) should require alignment — a member aligned by bridging joins, one amplified turns out, one transformed leads. Grumbles = residual misalignment, repairable in meetings (extension/amplification), dangerous in actions (a misaligned member doesn't show).

## Finding 3: The Decision Rule Is a Choice (Polletta, *Freedom Is an Endless Meeting*, 2002)

A century of American participatory democracy — labor educators, SNCC, SDS, feminist collectives, affinity groups, global-justice spokescouncils — and the consistent finding: **the failures aren't from democracy itself, they're from modeling democracy on the wrong relationships** (friendship → exclusivity and aversion to rules → splintering; tutelage → quiet hierarchy).

- **Consensus is double-edged.** It produces high *commitment* — a consensed action holds people because everyone consented to risk — and it's the natural rule for high-risk direct action (civil disobedience, affinity groups). Its costs: slow, favors the stubborn, and scales badly. GLF's open consensus meetings "became paralyzed" and splintered; Quaker-pedigreed groups ran it well because it was a practiced discipline, not a default.
- **Majority voting scales** and settles contested calls cleanly — the labor/union tradition — but manufactures a permanent minority (our `dissenters` list made literal).
- **Spokescouncils** are the scaling answer: small cells each send a mandated speaker — participation preserved past the size where one meeting breaks. Org structure as a tech-tree unlock.
- **Standing rules matter more than charisma.** SNCC/Ella Baker's "participatory democracy" worked because norms (who speaks, how proposals form, how dissent is heard) were explicit — the same lesson as Freeman, arrived at from history instead of theory.

**Mechanic insight:** a `DecisionRule` field on the campaign — Majority (fast, creates dissenters) vs Consensus (slow, heals grumbles, boosts action commitment, splinters above a size threshold without cells). Later: cells/spokescouncil as the structure that lifts the cap.

## Finding 4: Training Schools Multiply Leaders (Highlander / Citizenship Schools)

The best-documented leadership-multiplication machine in the record:

- **The answers come from the people.** Highlander's pedagogy taught from participants' own experience — a teacher was a "noncharismatic" community member, not an expert. Septima Clark's insight: the people best placed to teach organizing are the ones who just learned it.
- **It hid.** The first citizenship school on Johns Island met in the back of a grocery store, out of hostile sight; Esau Jenkins ran literacy lessons on his bus. Training infrastructure can be clandestine.
- **It replicated.** Students went home and started their own schools — 37 citizenship schools on the Sea Islands by 1961; transferred to SCLC and scaled to ~100,000 adults trained through the 1960s. Alumni founded housing projects, credit unions, nursing homes. Horton: "educational work during social movement periods provides the best opportunity for multiplying democratic leadership."
- **It was a bottleneck and a target.** Tennessee revoked Highlander's charter on pretext charges in 1961 — the state understood that killing the school hurt more than arresting any organizer.

**Mechanic insight:** the organizing skill tree shouldn't just grow by participation — it should *replicate by teaching*. `train` actions (a trainer whose skill clears a bar teaching students whose stake motivates them) is how `Organizing` skills spread through the org — and how they survive a leader's arrest. A training venue (a back room, a sympathetic church) is a campaign asset worth a prisoner's ransom.

## The Synthesis

Internal structure is a three-way dial:

1. **Formality** — seats and rules (Freeman). Too little: shadow elite. Explicit: accountable delegation.
2. **Frame alignment** — ongoing, per-member, repairable in meetings (Snow et al.). It's the hidden stat behind every turnout roll.
3. **Decision rule** — consensus vs majority vs councils (Polletta). Speed, dissent, and splinter-risk dials.

And training is the multiplier on all of it (Highlander): skills replicate through teaching faster than through participation alone, which is why organizers spend meetings training instead of acting — the base-building vs action trade-off made mechanical.

## Game Mechanic Proposals

### Mechanic 1: Alignment as the participation gate *(C1/C2/C4)*
`alignment(member, campaign) ∈ {Bridged, Amplified, Transformed}` derived from issue-stake match + framing fit + meetings attended. Turnout rolls and strike votes read alignment the way combat reads morale — a Bridged member votes and shows; Amplified recruits others; Transformed holds seats and leads actions. `grumbles` flags decompose into alignment shortfalls.

### Mechanic 2: Seats as accountability *(C2)*
Roles (NoteTaker, TurnoutCoordinator, Chair) exist only if delegated by vote — and an undelegated function doesn't go undone, it accrues to whoever has the strongest ties (the shadow elite). Track `tie_mass(member)` (sum of their relationship strengths to members): if a non-leader's tie_mass exceeds the leader's for N meetings, loyalty follows them — meetings start drifting to their agenda, and a contested vote can split the org. Delegation is how you keep gravity on the formal structure.

### Mechanic 3: Decision rules *(C2)*
`DecisionRule::Majority` (resolve in one meeting, dissenters recorded — current bones) vs `Consensus` (takes extra meeting rounds per unresolved grumble; passing clears grumbles and adds a commitment bonus to the next action; above ~12 members without cells, adds a splinter roll per contested decision). The organizer picks at founding; switching later costs a contested vote of its own.

### Mechanic 4: Frame work as a meeting action *(C2, deepens C1)*
Meetings get an agenda slot: **amplification** (voice attendees' stakes — converts grumbles to alignment; the current meeting-reveal machinery already does the front half) and **extension** (re-present a scoped issue so adjacent outsiders can vote it next time — a mason-scoped issue becomes village-scoped in the telling). This is "the organizer coaches the room" made concrete.

### Mechanic 5: Train-the-trainer *(C3)*
`teach(trainer, student, skill)` — requires trainer's skill ≥ Apprentice-equivalent; student's gain scaled by whether the campaign's issue matches their stake (motivation: the answers come from the people). Trainers grow `training` skill and the org gains capacity — multiplied leadership *is* organized power. Skills on the person persist across campaigns (already the plan); teaching is how they propagate.

### Mechanic 6: Hidden schools *(C3/C6 hook)*
A training venue can be secret (Clark's grocery-store school): meetings held under cover don't raise the org's public profile but train slower. When C6 repression lands, open venues are what the state shutters first — Highlander was killed by charter revocation, not truncheons.

## Key Sources

- **Freeman, "The Tyranny of Structurelessness"** (1970 talk; *The Second Wave* 1972; Berkeley Journal of Sociology 17) — A+. The canonical primary text on informal elites; every mechanic for delegation traces here.
- **Snow, Rochford, Worden & Benford, "Frame Alignment Processes, Micromobilization, and Movement Participation," *ASR* 51(4), 1986** — A+. 6,500+ citations; the four-process taxonomy is the standard model of why people join.
- **Polletta, *Freedom Is an Endless Meeting* (U. Chicago, 2002) + "Consensual decision-making," Wiley-Blackwell Encyclopedia of Social and Political Movements** — A. The authoritative history of American participatory democracy; the consensus-vs-majority and friendship-model failure analysis.
- **King Institute (Stanford), Highlander Folk School records** — A (primary-adjacent institutional history; Horton's "multiplying democratic leadership" quote, SCLC transfer, ~100k figure).
- **"Local Leadership in the Early Civil Rights Movement," *Journal of American History* (2011)** — A. The CEP/Sea Islands replication numbers and the Dorchester teacher-training pipeline.
- **"The Answers Come from the People," *History of Education Quarterly*** — A. Highlander's pedagogy read as communities-of-practice formation; the motivation-scaling basis for Mechanic 5.

## Next Steps

- Deepens `campaign-plan.md`: C2's cadence/roles now has the accountability model (Mechanic 2) and the decision-rule dial (Mechanic 3); C3's training gets the replication model (Mechanic 5); C4's turnout rolls should read alignment, not raw relationship strength alone.
- A companion survey on **prison organizing and release campaigns** (hunger strikes, jail solidarity, political prisoners as campaigns — Suffragette "Night of Terror," Attica, Amnesty's prisoner-of-conscience model) would complete C6's research base; arrests-as-recruitment is covered in campaign-masters, the inside is not.
- Open question: how to surface `alignment` without making the hidden numbers legible — structure tests are the only gauge (per Gary's rule). Alignment should leak through *behavior* (who shows, who grumbles) never through a meter.
