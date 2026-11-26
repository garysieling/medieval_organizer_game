# Playtest round 2 — #1621

Seed 42, `talk_lab` sandbox (real `Game`). All ten round-1 defect tickets
(#1622–#1630) and the two confirmed openers (#1610, #1615) had merged since
round 1 — this run plays the fixed build. Transcripts: `evidence/playtest-courting-*.txt`,
`evidence/playtest-meeting-*.txt`, `evidence/playtest-batch.txt`.

## Played

- Honest courting: Eliza + Bram, greet/scout/probe/leave across two in-game days.
- Meeting arc (staged via `--stage strongorg --plan`, real session code):
  word goes out, 9 gather at the square, airing round, floor steer, closer,
  organizer's call, cadence pick.
- `batch -n 8` for villager-villager diversity.

## Rubric

**1. Are conversations logical? Mostly yes.** Greets land, replies track the
beat, asks parse as asks. Two cracks:

- After the probe answer ("I'd like to feel safe, I'd warrant"), the ask row
  still reads "- for 6g" — the earned variant doesn't replace the priced one
  until the ask list's next refresh beat. (Filed.)
- The meeting's resolve prose says "They'd gather again - or take it further"
  but the call offers only "Meet again / Let it rest" — no "take it further"
  option ever appears. (Filed.)

**2. Do meeting portions make sense? Yes — verified fixed.** Round 0 airs nine
*distinct* grievances, each matching the speaker's make (Marga the pious →
"walk the boundary markers on Rogation days"; Nell → "Stand together on
harvest morning, or take starvation wages alone"; Aldric → the miller's take).
The steer menu ("Press it / Open the floor / Name the next step") matches the
act the prompt asks for — the #1624 pool-split holds in play. Closer
("Go in peace - and keep what was said here") and call follow cleanly.

**3. Diversity — improved, with residuals.** Cards now vary per villager
(#1629 verified: Bram's luthier card vs. Eliza's farmer card share no prose).
Scout answers vary by speaker (#1605 verified: "Folk keep their own counsel
here" vs. "Nobody's word carries far here - every house minds its own").
Residuals filed:

- Same-day repeat across talks: Eliza's "These damp mornings set my knees
  aching something awful" in talk 1 AND talk 2 on day 0 — a cross-talk
  freshness gap.
- Near-duplicate stems in one talk: "The calves won't feed themselves - and
  I'd not ask thee for help, Hob" then "The calves won't feed themselves,
  fever or no" — exact-match dedup can't see the shared stem.
- Stacked idiolect tags: "as I've cause to say, or so I make it" in one line.
- "Morrow, wanderer - a fine day to be out" opened Eliza's, Bram's and a
  second-day talk alike — the greet pool's breadth, not a bug per se, but
  noticeable.

**4. Culture teaching — the strongest axis.** Rogation boundary-walking,
enclosure ("the ditch enclosing the common wood has been unlawfully fenced
off"), starvation wages vs. standing together, the wives' petition to the
castle, the Merchant Guild forming, the smithy standing derelict — the world
teaches itself through mouths.

**5. Trade facts — thin.** Bram names the journeyman trap ("No one to learn
from. I'll be a journeyman forever") and the card shows Craft 15% —
personality, not practice. Still mostly vibes over substance.

**6. Teaches organizing — systems yes; legibility now catching up.** The probe
answer files on the ledger the same day (#1625 verified: card reads "an
acquaintance · 1 of 3 heard" and ISSUES lists "wants better tools"). But the
playtest exposed the last leg: the box's few slots went to Scout/Probe/coin
every beat — an earned ask never surfaced. Fixed in this PR: earned asks now
lead the ask list ahead of nerve/coin; `card` shows the next rung's gap.

**7. Show-don't-tell — largely yes.** Nothing lectures; the mechanics surface
through what folk say and what the card lets you read.

## Fixed in this PR (found during the round)

- `ask_offers` now sorts Earned before Nerve/Coin — the earned rung leads the
  ask list (`test_the_earned_asks_lead_the_asks_list`).
- talk_lab sandbox: `pick <text>`, `card`, `meet`, `--plan` (the courting
  yield, staged) — the harness this loop runs on.

## Defects filed

(see issue links in the #1621 comment — same-day ambient repeats, call-prose
vs. options mismatch, near-duplicate stems + stacked idiolect tags, stale
ask row after the probe lands)

## REPRO

```
cargo run -p game2-core --bin talk_lab -- sandbox --seed 42
  sel 1; talk; frame 400; pick actually want; card; day 1; ...
cargo run -p game2-core --bin talk_lab -- sandbox --seed 42 --stage strongorg --plan
  meet; frame 3000; pick open the floor; ...
cargo run -p game2-core --bin talk_lab -- batch -n 8 --seed 42
```
