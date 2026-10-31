---
agent: devin-local
created: 2026-10-31
status: ordered backlog — work top-down, one feature per PR
---
# Campaigns — organizing a village around an issue

The second rung of the organizing ladder. `organizing.rs` already has
1-1s, house meetings, and the `Organizer` ledger that surfaces issues;
`organized_power.rs` has the capacity that grows with good use and
atrophy idle; `relationships.rs` has per-person bonds that strengthen
with contact and dwindle without it. This work turns a group of people
who *share a grievance* into a *campaign* — a body that votes on its
focus, meets on a cadence, delegates roles, tests itself in public, gets
answered by the powers that be, and sometimes watches its own led away
in chains.

Gary's framing notes this round (bones now, deepen after lit research
lands):

- A group that generally agrees on an issue can build a campaign around
  it. Founding is a **vote** — the org agrees what to focus on.
- People vote their **self-interest**, and complain if the issue isn't
  phrased/framed quite right.
- The organizer chooses what to recommend: **meeting cadence** and the
  **delegation of roles** (note taker, turnout coordination).
- Campaigns ebb and flow. **Organizing is a separate skill tree** from
  vocations — 1-1s, house meetings, trainings for base-building vs.
  trainings for action. Learned by doing; **a skill earned on one
  campaign carries to the next** because it lives on the person.
- Relationships are per-person, grow with contact, dwindle slowly
  (-.01/turn vs +a good interaction). **Strength is secret from the
  player** — certainty only comes from tests. Coordinating a public
  action *is* the test of the org's strength.
- Public actions: a protest outside a building, a strike vote (X% of
  this vocation said yes — under ~90% is a failure), a townhall where a
  target PMC member is invited and demands are presented. The target
  answers by the scale of the action and their own interests — mostly
  they blow you off, which makes people feel helpless; the organizer's
  job is to coach people through that.
- **The action is in the reaction.** Actions move the views of
  non-participants who see them. Other groups — and the PMC — respond:
  the king may send knights to guard the streets.
- **The dungeon.** The organizer or members can be arrested. A campaign
  can be built just for someone's release. Bystanders sympathetic to the
  issue take ownership of it out of their own self-interest.

## How to work the list

Same rules as `feature-roadmap.md`: one feature = one PR, additive
`game/` modules, `cargo test --lib` green, don't disturb the text game
or `jobs.rs` users, check pending `feat/*` branches for overlap first.
`feat/dungeon` exists on the remote — check it before C6.

---

## C1 — Founding: the issue vote (`game/campaign.rs`)

An org that has discovered issues holds a vote to pick its fight.

- `struct Campaign { issue: IssueTag, framing: Framing, org: Entity,
  founded_day: u64, dissenters: Vec<Entity> }` — its own entity, like
  `Organization`/`Relationship`.
- `enum Framing { SharedGrievance, CommonFoe, MoralDuty }` — the
  organizer's choice of how to put the cause. A voter who wanted a
  different issue grumbles; so does one whose grievance doesn't suit the
  framing (a wronged villager wants a foe named; a broke one wants a
  shared burden shared).
- `fn hold_issue_vote(world, org, candidates: &[IssueTag], framing,
  day) -> Option<(Entity, IssueVote)>` — each living member votes the
  candidate closest to their live problems (`desires::problems_of` +
  `issue_tag`), else their unmet wants, else abstains. Plurality wins;
  ties go to the candidate the most members actually suffer. Result:
  the `Campaign` entity + ballots for the log.
- Self-interest is deterministic — no RNG. The drama is in the
  dissenters and the framing grumbles, which later pieces can act on
  (a dissenter skips meetings; a well-framed campaign recruits faster).

## C2 — Meetings: cadence and roles

A campaign that doesn't meet isn't one.

- `struct CampaignMeeting { cadence_days: u64, last_held: u64 }` on the
  campaign; organizer recommends a cadence, members ratify it at the
  founding vote (bones: cadence is a field, ratified vote is a count).
- `enum Role { NoteTaker, TurnoutCoordinator, Chair }` +
  `struct Seat { role: Role, holder: Entity }` — the organizer
  delegates; holding a role is what grows organizing skill (C3).
- `fn hold_campaign_meeting(...)`: turnout = members whose relationship
  with the TurnoutCoordinator clears a bar — the first real *structure
  test*, reading hidden relationship strength without showing it.
  Notes keep a meeting's resolves for later checks. Missing cadence
  ticks `OrganizedPower::idle_ticks` harder than usual (an org that
  skips its own meetings rots faster).

## C3 — Organizing skill tree (`game/org_skills.rs`)

A parallel `Craft`: `struct Organizing { one_on_one: f32, house_meeting:
f32, turnout: f32, agitation: f32, training: f32 }`, all 0..1, all grown
by participation — running a 1-1 grows `one_on_one`, holding a seat
grows the seat's skill, coaching through a blow-off grows `agitation`.
Lives on the person, so a veteran of one campaign starts the next one
dangerous. Trainings are a meeting agenda choice: a meeting spent
**training base-building** (1-1s, house meetings) vs **training for
action** (turnout, agitation) — the organizer's recurring trade-off.
Skill gates what an organizer can attempt, the way `level_of` gates
vocation actions.

## C4 — Structure tests: public actions (`game/actions.rs`)

- `enum PublicAction { Demonstration { at: Entity }, StrikeVote {
  vocation: Vocation }, Townhall { target: Entity } }`.
- Turnout for a demonstration reads each member's real relationship
  strength to the org/campaign — the number that comes back *is* the
  test result; the player learns strength by watching who shows up.
- `StrikeVote`: ballots of every villager with that `Craft.vocation`;
  yes = self-interest (does the campaign's issue bite them?) pulled by
  relational ties to members. ≥90% yes = a strike; less = a public
  failure that costs capacity (`Overreach` if the org hadn't built to
  it).
- `Townhall`: demands presented to a PMC target. Response by turnout
  scale vs. the target's interests (a collector won't carry tax demands
  to the lord; a priest might carry a moral one). Default response is
  the blow-off — members gain a `Helpless` grievance; the organizer
  spends a coaching action to clear it before it rots the rolls.

## C5 — Reactions (`game/reactions.rs`)

The action is in the reaction. A public action is seen: bystanders in
range update `InnerLife` (sympathy for the campaign's issue, or fear);
factions and PMC answer — the king's answer to a real demonstration is
knights posted to the streets (reuse `military.rs` units as posted
guards). Reactions feed back: a crackdown is itself a grievance that
recruits (`found_issue` on a "the crackdown" tag).

## C6 — The dungeon (`game/dungeon.rs`)

`struct Imprisoned { since: u64, charge }` — arrests come out of C5
backlash (the loudest member, or the organizer). An imprisoned member
can't meet or act; leadership already passes via `living_leader`. A
release campaign is just a `Campaign` whose issue is "the prisoners" —
bystanders whose self-interest touches it (family of the jailed, the
sympathetic) can join without ever having been organized: the issue
organizes them.

---

## Suggested PR order

1. This document.
2. C1 `game/campaign.rs` — vote, framing, dissenters + unit tests.
3. C2 cadence + roles + turnout test.
4. C3 `game/org_skills.rs` — skills grow from participation; persist.
5. C4 `game/actions.rs` — demonstration/strike/townhall.
6. C5 `game/reactions.rs` — bystanders, factions, the king's knights.
7. C6 `game/dungeon.rs` — arrests, imprisonment, release campaigns.
8. Wiring pass: `game_ui` buttons/inspector + text-game verbs, once the
   bones prove out — same late-wiring rule as the vocation work.
