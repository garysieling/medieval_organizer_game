# The Village's Own Constitution - Design Principles for Future Decisions

The game's own north star, set down so the agents and the contributors
reach for the same answers. When a mechanic, a name, or a system is in
question, weigh it against these.

## 1. Medieval as a Set Piece

"Medieval" is a set piece, not a period lock. Historical
time-blindness in the style of Bridgerton - anachronisms are fine, but
nothing purely modern (post-Victorian) should appear. Translate modern
concepts into time-appropriate ones that existed between AD 500 and AD
1800. A "union" is a guild or a confraternity; a "campaign" is a cause,
a petition drive, a rising.

## 2. System Effects over Win/Loss (Victoria 3)

The player doesn't win or lose - they have major effects on the
system. The sim's own emergent pressure is the game: capacity,
agitation, grievances, the org's own strength vs the powers' own arm.
The effects should be MORE pronounced than Victoria 3 - the player's
own organizing is the whole thesis, not one system among many.

## 3. Generated Storyline (RimWorld)

The game's a generated storyline - periods of grinding building up to
major events of varying sizes, some from the player's own actions and
some random. The `arc_phase`/`storyteller`/`drought`/`event_pace`
machinery carries this: the setup looms, the escalation builds, the
climax fires, the resolution rests. Pacing's a conditioning schedule,
not a constant pressure.

## 4. No Direct Control (RimWorld + Majesty)

The player doesn't control the characters - they organize, influence,
and answer. The villagers are their own agents (the `Routine`/`Goal`/
`travel` machinery); the player's own reach is the talk, the recruit,
the call, the cause - never the click-to-move. The `drive_agent`
policy's the bot half of the same rule.

## 5. Resource Chains (Against the Storm)

Multiple resources making higher-level goods - the constraints aren't
inherently limiting, they're the pressure's own shape. The
`Vocation`/`output`/`inputs`/`luxuries` machinery carries this: the
crafts chain the raw to the finished, the tranches land the luxuries.

## 6. Teach Historical Concepts

The game's a teaching instrument - the organizing's own history (the
IAF, the faith-power arc, the movement-history corpus in
`wiki/literature-reviews/`) grounds the mechanics. A new mechanic
should name the real thing it's teaching - the paternalism bargain,
the Constantinian trap, the dissolution-and-self-organization - not an
abstract system.

## 7. Legible, Not Fair

The game can be unfair; it can never be illegible. A blow arrives with
a herald - the writ rides before the knights, the cough warns before
the fever. A verb says what it does and what it costs before it's
pressed ("Raise the schoolhouse (30)" - #1071). And a failure still
reads: the man who promised the hall and never came is named for it
("Bram was to open us, but he never came" - #1076). A consequence the
player can't trace to something they saw isn't difficulty - it's a bug.
The `punishment_warn` KPI already enforces the herald half of this.

## 8. Verbs Are Earned

The player's reach is built through relationships and world-state, not
granted by interface privilege - "earn the house meeting: organize
through relationships, not a button" (#473). An offer gates on where
the player stands, who they know, what's been promised; a new verb
arrives when the world agrees it should, not when the UI decides to
show it.

## 9. The Village Speaks, and the Interface Tells the Truth

The player learns the world through people, signs and words - rumor,
gossip, the look of a place - not through omniscient readouts; the raw
numbers stay probe-side (the public/secret KPI split in
`session/kpi/`). And what the interface does show is true: drawn
numbers are real keys (#1060), a label carries its cost, and a blocked
action either says why or isn't there - grey-with-reason when the
reason itself is worth the player's learning (#1112), absent when the
verb isn't theirs to reach for (#1071).

---

*Written for the ticket loop and the grill pass - weigh a proposal
against these, and flag the one it breaks.*
